use anyhow::Result;
use pe_disector::image::*;
use std::path::Path;

fn main() -> Result<()> {
    let sample_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples\\");

    let mut dll = Image::from_path(sample_path.join("sample1.dll"))?;

    let nt_header = dll.get_nt_header();

    let _ = dll.get_section_table();

    Ok(())
}
