//! 进程内 stdout/stderr 捕获（N-API 宿主专用；Windows 上 `gag` 与 Node 冲突）。

use std::io::Read;

/// 运行闭包并捕获 stdout/stderr 文本。
pub fn capture<F>(run: F) -> (i32, String, String)
where
    F: FnOnce() -> i32,
{
    #[cfg(windows)]
    {
        return win::capture(run);
    }
    #[cfg(not(windows))]
    {
        unix::capture(run)
    }
}

#[cfg(windows)]
mod win {
    use super::*;
    use std::io::{self, pipe};
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::System::Console::{GetStdHandle, SetStdHandle, STD_ERROR_HANDLE, STD_OUTPUT_HANDLE};

    pub fn capture<F>(run: F) -> (i32, String, String)
    where
        F: FnOnce() -> i32,
    {
        let orig_stdout = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
        let orig_stderr = unsafe { GetStdHandle(STD_ERROR_HANDLE) };
        let (stdout_read, stdout_write) = pipe().expect("stdout pipe");
        let (stderr_read, stderr_write) = pipe().expect("stderr pipe");
        set_std_handle(STD_OUTPUT_HANDLE, stdout_write.as_raw_handle());
        set_std_handle(STD_ERROR_HANDLE, stderr_write.as_raw_handle());

        let status = run();

        set_std_handle(STD_OUTPUT_HANDLE, orig_stdout);
        set_std_handle(STD_ERROR_HANDLE, orig_stderr);
        drop(stdout_write);
        drop(stderr_write);
        let stdout = read_pipe(stdout_read).unwrap_or_default();
        let stderr = read_pipe(stderr_read).unwrap_or_default();
        (status, stdout, stderr)
    }

    fn set_std_handle(kind: u32, handle: std::os::windows::io::RawHandle) {
        unsafe {
            SetStdHandle(kind, handle as HANDLE);
        }
    }

    fn read_pipe<R: Read>(mut reader: R) -> io::Result<String> {
        let mut buffer = String::new();
        reader.read_to_string(&mut buffer)?;
        Ok(buffer)
    }
}

#[cfg(not(windows))]
mod unix {
    use super::*;

    pub fn capture<F>(run: F) -> (i32, String, String)
    where
        F: FnOnce() -> i32,
    {
        let mut stdout_redirect = gag::BufferRedirect::stdout().expect("redirect stdout");
        let mut stderr_redirect = gag::BufferRedirect::stderr().expect("redirect stderr");
        let status = run();
        let mut stdout = String::new();
        let mut stderr = String::new();
        let _ = stdout_redirect.read_to_string(&mut stdout);
        let _ = stderr_redirect.read_to_string(&mut stderr);
        (status, stdout, stderr)
    }
}
