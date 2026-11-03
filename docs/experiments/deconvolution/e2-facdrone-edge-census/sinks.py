"""E2 frozen effect-sink list (plan step 2). Data only; dotted import paths resolved by the recorder.

A call counts as an effect only when the caller is Facdrone production code and the callee is
identical to one of these objects. Anything not listed is not counted, and that is reported.
"""

SINKS = (
    # files
    "builtins.open", "io.open", "pathlib.Path.open", "pathlib.Path.write_text",
    "pathlib.Path.write_bytes", "os.replace", "os.open", "os.fdopen",
    # json.dump*
    "json.dump", "json.dumps",
    # pickle
    "pickle.dump", "pickle.dumps", "pickle.load", "pickle.loads",
    # subprocess
    "subprocess.run", "subprocess.Popen", "subprocess.call", "subprocess.check_call",
    "subprocess.check_output",
    # socket
    "socket.socket", "socket.create_connection", "socket.getaddrinfo",
)

# SQLAlchemy cursor execution is captured by a class-level `before_cursor_execute` listener on
# sqlalchemy.engine.Engine, not by call identity.
SQL_SINK = "sqlalchemy.cursor_execute"
