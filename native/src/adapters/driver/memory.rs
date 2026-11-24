//! How much memory the system can still hand out, as the system says it.
//! Unknown on a platform this file has no reading for; nothing is guessed.

/// Bytes of memory that are available now, if the platform tells.
pub fn available() -> Option<u64> {
    system::available()
}

#[cfg(windows)]
mod system {
    /// `MEMORYSTATUSEX` of the Windows API.
    #[repr(C)]
    struct Status {
        length: u32,
        load: u32,
        total_physical: u64,
        available_physical: u64,
        total_page_file: u64,
        available_page_file: u64,
        total_virtual: u64,
        available_virtual: u64,
        available_extended_virtual: u64,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GlobalMemoryStatusEx(status: *mut Status) -> i32;
    }

    /// The smaller of the physical memory that is free and what can still be
    /// committed: a build fails at whichever runs out first.
    pub fn available() -> Option<u64> {
        let mut status = Status {
            length: std::mem::size_of::<Status>() as u32,
            load: 0,
            total_physical: 0,
            available_physical: 0,
            total_page_file: 0,
            available_page_file: 0,
            total_virtual: 0,
            available_virtual: 0,
            available_extended_virtual: 0,
        };
        // SAFETY: the structure is ours, of the size its first field states,
        // and the call only writes into it.
        let told = unsafe { GlobalMemoryStatusEx(&mut status) };
        (told != 0).then(|| status.available_physical.min(status.available_page_file))
    }
}

#[cfg(target_os = "linux")]
mod system {
    pub fn available() -> Option<u64> {
        let text = std::fs::read_to_string("/proc/meminfo").ok()?;
        super::of_meminfo(&text)
    }
}

#[cfg(not(any(windows, target_os = "linux")))]
mod system {
    pub fn available() -> Option<u64> {
        None
    }
}

/// `MemAvailable` of `/proc/meminfo`, in bytes.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub fn of_meminfo(text: &str) -> Option<u64> {
    let line = text.lines().find(|l| l.starts_with("MemAvailable:"))?;
    let kib: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kib * 1024)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_available_line_of_meminfo_is_read_in_bytes() {
        let text = "MemTotal:       32000000 kB\nMemFree:         1000000 kB\nMemAvailable:    8388608 kB\n";
        assert_eq!(of_meminfo(text), Some(8_388_608 * 1024));
        assert_eq!(of_meminfo("MemTotal: 1 kB\n"), None);
    }

    #[cfg(any(windows, target_os = "linux"))]
    #[test]
    fn this_system_tells_how_much_is_available() {
        assert!(available().is_some_and(|bytes| bytes > 0));
    }
}
