static mut VARIABLE: usize = 0x10000;

fn main() {
    unsafe {
        println!("{:X?}", (&raw const VARIABLE).as_ref_unchecked());

        *(&raw mut VARIABLE).as_mut_unchecked() += 0x500;

        println!("{:X?}", (&raw const VARIABLE).as_ref_unchecked());
    }
}
