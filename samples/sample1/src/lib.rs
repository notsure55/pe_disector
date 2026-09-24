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
use windows::Win32::Foundation::GENERIC_WRITE;
use windows::Win32::Foundation::{HINSTANCE, HMODULE};
use windows::Win32::Storage::FileSystem::{
    CreateFileA, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows::Win32::System::Console::{AllocConsole, FreeConsole, SetStdHandle, STD_OUTPUT_HANDLE};
use windows::Win32::System::LibraryLoader::FreeLibraryAndExitThread;
use windows::Win32::System::Threading::{CreateThread, THREAD_CREATION_FLAGS};
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
use windows_core::s;

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
    unsafe {
        let _ = AllocConsole();

        let con_out = CreateFileA(
            s!("CONOUT$"),
            GENERIC_WRITE.0,
            FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )
        .unwrap();

        let _ = SetStdHandle(STD_OUTPUT_HANDLE, con_out);
    }

    loop {
        if unsafe { GetAsyncKeyState(0x75) } < 0 {
            break;
        }

        println!("We are inside!");
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    cleanup(h_module);
}

pub fn cleanup(h_module: HMODULE) {
    unsafe {
        let _ = FreeConsole();
        let _ = FreeLibraryAndExitThread(h_module, 0);
    }
}
