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
