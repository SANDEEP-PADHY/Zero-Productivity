use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows::Win32::Foundation::{CloseHandle, HWND, MAX_PATH};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
};

pub fn get_window_title(hwnd: HWND) -> Option<String> {
    unsafe {
        let length = GetWindowTextLengthW(hwnd);
        if length == 0 {
            return None;
        }

        let mut buffer: Vec<u16> = vec![0; (length + 1) as usize];
        let copied = GetWindowTextW(hwnd, &mut buffer);
        if copied == 0 {
            return None;
        }

        Some(OsString::from_wide(&buffer[..copied as usize]).to_string_lossy().into_owned())
    }
}

pub fn get_process_id_from_hwnd(hwnd: HWND) -> u32 {
    let mut pid: u32 = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
    }
    pid
}

pub fn get_process_path(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        
        let mut buffer = [0u16; MAX_PATH as usize];
        let mut size = MAX_PATH;
        
        let result = windows::Win32::System::Threading::QueryFullProcessImageNameW(
            handle,
            windows::Win32::System::Threading::PROCESS_NAME_FORMAT(0),
            windows::core::PWSTR(buffer.as_mut_ptr()),
            &mut size,
        );
        
        CloseHandle(handle).ok();

        if result.is_ok() {
            Some(OsString::from_wide(&buffer[..size as usize]).to_string_lossy().into_owned())
        } else {
            None
        }
    }
}

pub fn get_process_name_from_path(path: &str) -> Option<String> {
    std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
}
