"""Private native Windows lifecycle helper; imported only on Windows.

The CPython Windows API wrapper supplies process/pipe handles. ctypes supplies
the Job Object APIs and ResumeThread missing from that wrapper. A job controls
ordinary workload lifetime; it is not a security sandbox.
"""

import _winapi
import ctypes
from ctypes import wintypes
import msvcrt
import os
from pathlib import Path
import subprocess
import time
from typing import BinaryIO, Literal

from .contracts import ResolvedCommand


class _BasicLimits(ctypes.Structure):
    _fields_ = [("PerProcessUserTimeLimit", ctypes.c_longlong),
                ("PerJobUserTimeLimit", ctypes.c_longlong),
                ("LimitFlags", wintypes.DWORD),
                ("MinimumWorkingSetSize", ctypes.c_size_t),
                ("MaximumWorkingSetSize", ctypes.c_size_t),
                ("ActiveProcessLimit", wintypes.DWORD),
                ("Affinity", ctypes.c_size_t),
                ("PriorityClass", wintypes.DWORD),
                ("SchedulingClass", wintypes.DWORD)]


class _IOCounters(ctypes.Structure):
    _fields_ = [(name, ctypes.c_ulonglong) for name in
                ("ReadOperationCount", "WriteOperationCount", "OtherOperationCount",
                 "ReadTransferCount", "WriteTransferCount", "OtherTransferCount")]


class _ExtendedLimits(ctypes.Structure):
    _fields_ = [("BasicLimitInformation", _BasicLimits), ("IoInfo", _IOCounters),
                ("ProcessMemoryLimit", ctypes.c_size_t), ("JobMemoryLimit", ctypes.c_size_t),
                ("PeakProcessMemoryUsed", ctypes.c_size_t),
                ("PeakJobMemoryUsed", ctypes.c_size_t)]


_kernel = ctypes.WinDLL("kernel32", use_last_error=True)
_kernel.CreateJobObjectW.argtypes = [ctypes.c_void_p, wintypes.LPCWSTR]
_kernel.CreateJobObjectW.restype = wintypes.HANDLE
_kernel.SetInformationJobObject.argtypes = [wintypes.HANDLE, ctypes.c_int,
                                           ctypes.c_void_p, wintypes.DWORD]
_kernel.SetInformationJobObject.restype = wintypes.BOOL
_kernel.AssignProcessToJobObject.argtypes = [wintypes.HANDLE, wintypes.HANDLE]
_kernel.AssignProcessToJobObject.restype = wintypes.BOOL
_kernel.ResumeThread.argtypes = [wintypes.HANDLE]
_kernel.ResumeThread.restype = wintypes.DWORD


def _create_job() -> int:
    job = _kernel.CreateJobObjectW(None, None)
    if not job:
        raise ctypes.WinError(ctypes.get_last_error())
    limits = _ExtendedLimits()
    limits.BasicLimitInformation.LimitFlags = 0x2000  # KILL_ON_JOB_CLOSE, no breakaway
    if not _kernel.SetInformationJobObject(job, 9, ctypes.byref(limits), ctypes.sizeof(limits)):
        error = ctypes.WinError(ctypes.get_last_error())
        _winapi.CloseHandle(job)
        raise error
    return job


def _assign_job(job: int, process: int) -> None:
    if not _kernel.AssignProcessToJobObject(job, process):
        raise ctypes.WinError(ctypes.get_last_error())


def _resume(thread: int) -> None:
    if _kernel.ResumeThread(thread) == 0xffffffff:
        raise ctypes.WinError(ctypes.get_last_error())


def run_windows(command: ResolvedCommand, cwd: Path, stdout: BinaryIO,
                stderr: BinaryIO, start: float) -> tuple[
                    Literal["completed", "timeout", "log_limit", "error"], int | None]:
    """Create containment first; never resume a process outside its configured job."""
    job: int | None = None
    process: int | None = None
    thread: int | None = None
    assigned = False
    handles: list[int] = []
    captured = 0
    status: Literal["completed", "timeout", "log_limit", "error"] = "completed"
    try:
        try:
            job = _create_job()
            out_read, out_write = _winapi.CreatePipe(None, 0)
            handles.extend((out_read, out_write))
            err_read, err_write = _winapi.CreatePipe(None, 0)
            handles.extend((err_read, err_write))
            with open(os.devnull, "rb") as null:
                null_handle = msvcrt.get_osfhandle(null.fileno())
                inherited = [null_handle, out_write, err_write]
                for handle in inherited:
                    os.set_handle_inheritable(handle, True)
                startup = subprocess.STARTUPINFO()
                startup.dwFlags = _winapi.STARTF_USESTDHANDLES | _winapi.STARTF_USESHOWWINDOW
                startup.wShowWindow = _winapi.SW_HIDE
                startup.hStdInput, startup.hStdOutput, startup.hStdError = inherited
                startup.lpAttributeList = {"handle_list": inherited}
                process, thread, _, _ = _winapi.CreateProcess(
                    command.argv[0], subprocess.list2cmdline(command.argv), None, None,
                    True, 0x4 | 0x08000000,  # CREATE_SUSPENDED | CREATE_NO_WINDOW
                    dict(command.environment), str(cwd), startup)
            for handle in (out_write, err_write):
                _winapi.CloseHandle(handle)
                handles.remove(handle)
            _assign_job(job, process)
            assigned = True
            _resume(thread)
            _winapi.CloseHandle(thread)
            thread = None
        except OSError as exc:
            stderr.write(("process_error: " + str(exc) + "\n").encode()[:command.limits.max_log_bytes])
            return "error", None

        streams = {out_read: stdout, err_read: stderr}
        cleanup_at: float | None = None
        exit_code: int | None = None
        while streams or exit_code is None:
            now = time.monotonic()
            if exit_code is None and _winapi.WaitForSingleObject(process, 0) == 0:
                exit_code = _winapi.GetExitCodeProcess(process)
            if cleanup_at is None:
                if status == "completed" and now - start >= command.limits.timeout_seconds:
                    status = "timeout"
                if status != "completed" or exit_code is not None:
                    # Closing the sole noninherited job handle kills all descendants.
                    _winapi.CloseHandle(job)
                    job = None
                    cleanup_at = now
            elif now - cleanup_at > .2:
                break
            for handle, target in list(streams.items()):
                try:
                    available = _winapi.PeekNamedPipe(handle, 0)[0]
                    data = _winapi.ReadFile(handle, min(available, 65536))[0] if available else b""
                except OSError as exc:
                    if exc.winerror not in (109, 232):  # broken/disconnected pipe
                        raise
                    del streams[handle]
                    continue
                if data:
                    remaining = command.limits.max_log_bytes - captured
                    retained = data[:remaining]
                    target.write(retained)
                    target.flush()
                    captured += len(retained)
                    if len(data) > remaining and status == "completed":
                        status = "log_limit"
            time.sleep(.005)
        _winapi.WaitForSingleObject(process, _winapi.INFINITE)
        return status, _winapi.GetExitCodeProcess(process)
    finally:
        if job is not None:
            _winapi.CloseHandle(job)
        if process is not None:
            # Also kills a suspended process when assignment/resumption failed.
            if not assigned and _winapi.WaitForSingleObject(process, 0) != 0:
                _winapi.TerminateProcess(process, 1)
            _winapi.WaitForSingleObject(process, _winapi.INFINITE)
            _winapi.CloseHandle(process)
        if thread is not None:
            _winapi.CloseHandle(thread)
        for handle in handles:
            _winapi.CloseHandle(handle)
