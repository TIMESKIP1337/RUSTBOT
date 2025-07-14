use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::PCWSTR;
use std::ffi::OsString;
use std::os::windows::ffi::OsStrExt; 
use log::{info, error};
use std::time::Duration;
use tokio::time::sleep;

pub fn find_scum_window() -> Option<HWND> {
    unsafe {
        let window_names = ["SCUM", "SCUM  ", "SCUM - Unreal Engine"];

        for name in &window_names {
            let wide_name: Vec<u16> = OsString::from(name)
                .encode_wide()
                .chain(Some(0))
                .collect();

            let hwnd = FindWindowW(None, PCWSTR(wide_name.as_ptr() as *const u16));
            if hwnd != HWND(0) {
                info!("Found SCUM window: '{}' (hwnd={:?})", name, hwnd);
                return Some(hwnd);
            }
        }

        error!("SCUM window not found");
        None
    }
}

pub async fn send_commands_to_game(commands: Vec<String>, command_type: &str) {
    let hwnd = match find_scum_window() {
        Some(h) => h,
        None => {  
            error!("Cannot send commands: SCUM window not found!");
            return;
        }
    };

    let (char_delay, enter_delay) = match command_type {  
        "destroy" => (5, 3), 
        _ => (3, 3), 
    };

    info!("Using delays: char_delay={}ms, enter_delay={}ms", char_delay, enter_delay);

    unsafe {
        for command in commands {
            let cmd = command.trim();
            if cmd.is_empty() {
                info!("Skipping empty command");
                continue;
            }

            info!("Sending command to SCUM ({}): {}", command_type, cmd);

            for ch in cmd.chars() {
                PostMessageW(hwnd, WM_CHAR, WPARAM(ch as usize), LPARAM(0));
                sleep(Duration::from_millis(char_delay)).await;
            }

            PostMessageW(hwnd, WM_KEYDOWN, WPARAM(0x0D), LPARAM(0));
            PostMessageW(hwnd, WM_KEYUP, WPARAM(0x0D), LPARAM(0));
            sleep(Duration::from_millis(enter_delay)).await;
        }
    }
}

pub fn substitute_steam_id_in_commands(commands: &[String], steam_id: &str) -> Vec<String> {
    commands.iter()
        .map(|cmd| cmd.replace("{steam_id}", steam_id))
        .collect()
}

pub fn is_special_command(cmd: &str, special_commands: &[String]) -> bool {
    special_commands.iter().any(|special| cmd.contains(special))
}

pub fn calculate_discounted_price(
    base_price: u32,
    quantity: u32,
    discount: f32
) -> (u32, u32, u32) {
    let original_total = base_price * quantity;
    let discounted_total = (original_total as f32 * (1.0 - discount)) as u32;
    let discount_percent = (discount * 100.0) as u32;

    (original_total, discounted_total, discount_percent)
}