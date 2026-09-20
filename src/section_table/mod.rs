use crate::convert_mut_slice_to_ptr;
use ref_mut_field::RefMutFields;
use windows_types::IMAGE_SECTION_HEADER;

mod section_header;
use section_header::SectionHeader;

#[derive(Debug, RefMutFields)]
pub struct SectionTable {
    #[ref_mut]
    headers: Vec<SectionHeader>,
}

impl SectionTable {
    pub fn from_bytes(bytes: &mut [u8], start: usize, section_count: usize) -> Self {
        let headers = unsafe {
            std::slice::from_raw_parts_mut(
                convert_mut_slice_to_ptr!(bytes => IMAGE_SECTION_HEADER, start),
                section_count,
            )
        };

        let headers: Vec<_> = headers
            .iter_mut()
            .map(|header| SectionHeader::from_bytes(header as *mut _, bytes))
            .collect();

        Self { headers }
    }
    pub fn find_section_by_name(&self, name: impl AsRef<str>) -> Option<&SectionHeader> {
        self.headers.iter().find(|header| {
            if header.name().expect("Failed to unwrap name of section") == name.as_ref() {
                true
            } else {
                false
            }
        })
    }
}
