use anyhow::Result;
use pe_disector::image::*;
use pe_disector::to_prim;
use std::path::Path;

fn main() -> Result<()> {
    let sample_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("target\\debug\\");

    let mut dll = Image::from_path(sample_path.join("sample1.dll"))?;

    let nt_header = dll.get_nt_header();

    Ok(())
}
