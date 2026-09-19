use anyhow::Result;
use pe_disector_v2::image::*;
use std::path::Path;

fn main() -> Result<()> {
    let sample_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples\\");

    let mut exe = Image::from_path(sample_path.join("sample.exe"))?;

    let nt_header = exe.get_nt_header();

    let optional_header = nt_header.get_optional_header();

    let section_table = exe.get_section_table();

    Ok(())
}
