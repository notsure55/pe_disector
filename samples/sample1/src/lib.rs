#[unsafe(no_mangle)]
pub extern "system" fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[unsafe(no_mangle)]
pub extern "system" fn sub(left: u64, right: u64) -> u64 {
    left - right
}

#[unsafe(no_mangle)]
pub extern "system" fn mul(left: u64, right: u64) -> u64 {
    left * right
}

#[unsafe(no_mangle)]
pub extern "system" fn div(left: u64, right: u64) -> u64 {
    left / right
}

use std::ffi::c_void;
use windows::Win32::Foundation::{HINSTANCE, HMODULE};
use windows::Win32::System::LibraryLoader::FreeLibraryAndExitThread;
use windows::Win32::System::Threading::{CreateThread, THREAD_CREATION_FLAGS};
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_OK};
use windows_core::w;

unsafe extern "system" fn entry(handle: *mut c_void) -> u32 {
    init(HMODULE(handle));
    0
}

#[unsafe(no_mangle)]
pub extern "system" fn DllMain(handle: HINSTANCE, reason: u32, _: usize) -> i32 {
    match reason {
        1 => unsafe {
            let _ = CreateThread(
                None,
                0,
                Some(entry),
                Some(handle.0 as *const c_void),
                THREAD_CREATION_FLAGS(0),
                None,
            );
        },
        _ => (),
    }
    1
}

pub fn init(h_module: HMODULE) {
    unsafe { MessageBoxW(None, w!("hi this is a box!"), w!("im your caption"), MB_OK) };

    cleanup(h_module);
}

pub fn cleanup(h_module: HMODULE) {
    unsafe {
        let _ = FreeLibraryAndExitThread(h_module, 0);
    }
}
