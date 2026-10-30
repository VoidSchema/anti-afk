//! Foreground window checks, so input only lands in the Roblox client.

use windows::core::PWSTR;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
};

/// Human readable summary of the window in front of the user, used by `--probe`.
pub fn describe_foreground() -> String {
    let exe = foreground_exe().unwrap_or_else(|| "<unknown>".to_string());
    let title = foreground_title().unwrap_or_else(|| "<none>".to_string());

    format!(
        "process : {exe}\ntitle   : {title}\nroblox  : {}",
        if roblox_focused() { "yes" } else { "no" }
    )
}

/// True when the window in front of the user belongs to Roblox.
pub fn roblox_focused() -> bool {
    let exe = foreground_exe().unwrap_or_default().to_ascii_lowercase();

    if exe.contains("robloxplayer") || exe.contains("robloxstudio") {
        return true;
    }

    // The Microsoft Store build runs inside a generic host process, so fall back
    // to the window title there.
    if exe.is_empty() || exe == "windows10universal.exe" || exe == "applicationframehost.exe" {
        return foreground_title()
            .map(|title| title.to_ascii_lowercase().contains("roblox"))
            .unwrap_or(false);
    }

    false
}

fn foreground_exe() -> Option<String> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_invalid() {
            return None;
        }

        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return None;
        }

        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buffer = [0u16; 512];
        let mut len = buffer.len() as u32;
        let result = QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut len,
        );
        let _ = CloseHandle(process);
        result.ok()?;

        let path = String::from_utf16_lossy(&buffer[..len as usize]);
        let name = path.rsplit(['\\', '/']).next().unwrap_or(&path).to_string();
        Some(name)
    }
}

fn foreground_title() -> Option<String> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_invalid() {
            return None;
        }

        let mut buffer = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut buffer);
        if len <= 0 {
            return None;
        }

        Some(String::from_utf16_lossy(&buffer[..len as usize]))
    }
}
