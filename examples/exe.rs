use anyhow::Result;
use pe_disector::image::*;
use std::path::Path;

fn main() -> Result<()> {
    let sample_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples\\");

    let mut exe = Image::from_path(sample_path.join("sample.exe"))?;

    let nt_header = exe.get_nt_header();

    let optional_header = nt_header.get_optional_header();

    let section_table = exe.get_section_table();

    let mut import_table = exe.get_import_table_mut().as_mut().unwrap();
    import_table.update_iat("KERNEL32.dll", "GetLastError", 0x100000);

    Ok(())
}
