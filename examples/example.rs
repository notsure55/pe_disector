use anyhow::Result;
//use pe_disector::pe::Image;
use pe_disector::*;
use std::path::Path;

fn main() -> Result<()> {
    let sample_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples\\");

    let dll = pe::ImageOwned::from_path(sample_path.join("sample.exe"))?;

    //let dll = pe::ImageOwned::from_path(sample_path.join("sample1.dll"))?;

    dbg!(dll);

    Ok(())
}
