use anyhow::Result;
use pe_disector::image::*;
use pe_disector::to_prim;
use std::path::Path;

fn main() -> Result<()> {
    let sample_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples\\");

    let mut dll = Image::from_path(sample_path.join("sample1.dll"))?;

    let nt_header = dll.get_nt_header();

    let base_reloc_table = dll.get_relocation_table().as_ref().unwrap();

    dbg!(&base_reloc_table);

    base_reloc_table.update_relocs(
        to_prim!(nt_header.get_optional_header().get_image_base() => Va),
        0x190000000,
    );

    dbg!(&base_reloc_table);

    Ok(())
}
