use anyhow::Result;
use pe_disector::*;
use pe_disector::{pe::Image, section::section_table};
use std::path::Path;

fn main() -> Result<()> {
    let sample_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples\\");

    let exe = pe::ImageOwned::from_path(sample_path.join("sample.exe"))?;

    dbg!(&exe.section_table());

    let bytes = exe.section_bytes();

    let section_table = section_table::SectionTableOwned::from_bytes(&bytes, 5, 0)?;

    dbg!(&section_table as &dyn section_table::SectionTable);

    //let dll = pe::ImageOwned::from_path(sample_path.join("sample1.dll"))?;

    Ok(())
}
