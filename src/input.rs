//! Thin wrappers around `SendInput` for key taps, mouse nudges and hotkey polling.

use std::mem::size_of;
use std::thread::sleep;
use std::time::Duration;

use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, MapVirtualKeyW, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE,
    KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE,
    MAPVK_VK_TO_VSC, MOUSEEVENTF_MOVE, MOUSEINPUT, MOUSE_EVENT_FLAGS, VIRTUAL_KEY,
};

pub const VK_F8: i32 = 0x77;
pub const VK_F9: i32 = 0x78;

/// Reports a key press once, the moment the key goes from up to down.
pub struct EdgeKey {
    vk: i32,
    was_down: bool,
}

impl EdgeKey {
    pub fn new(vk: i32) -> Self {
        Self {
            vk,
            was_down: false,
        }
    }

    pub fn just_pressed(&mut self) -> bool {
        let down = unsafe { GetAsyncKeyState(self.vk) } as u16 & 0x8000 != 0;
        let pressed = down && !self.was_down;
        self.was_down = down;
        pressed
    }
}

/// Presses and releases a key using its hardware scan code, which is what games read.
pub fn tap_key(vk: u16, hold: Duration, extended: bool) -> Result<(), String> {
    let scan = unsafe { MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC) } as u16;
    if scan == 0 {
        return Err(format!("no scan code for virtual key 0x{vk:02X}"));
    }

    let mut flags = KEYEVENTF_SCANCODE.0;
    if extended {
        flags |= KEYEVENTF_EXTENDEDKEY.0;
    }

    send_key(scan, KEYBD_EVENT_FLAGS(flags))?;
    sleep(hold);
    send_key(scan, KEYBD_EVENT_FLAGS(flags | KEYEVENTF_KEYUP.0))?;
    sleep(Duration::from_millis(40));

    Ok(())
}

fn send_key(scan: u16, flags: KEYBD_EVENT_FLAGS) -> Result<(), String> {
    let input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    let sent = unsafe { SendInput(&[input], size_of::<INPUT>() as i32) };
    if sent != 1 {
        return Err("SendInput rejected the keyboard event".to_string());
    }

    Ok(())
}

/// Moves the cursor by a few pixels, then puts it back.
pub fn nudge_mouse(dx: i32, dy: i32, pause: Duration) {
    move_mouse(dx, dy);
    sleep(pause);
    move_mouse(-dx, -dy);
}

fn move_mouse(dx: i32, dy: i32) {
    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx,
                dy,
                mouseData: 0,
                dwFlags: MOUSE_EVENT_FLAGS(MOUSEEVENTF_MOVE.0),
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    unsafe {
        SendInput(&[input], size_of::<INPUT>() as i32);
    }
}
