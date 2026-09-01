use crate::arch;
use crate::windows_types::*;

#[derive(Debug)]
pub struct FileHeader(IMAGE_FILE_HEADER);

impl FileHeader {
    pub fn new(raw: IMAGE_FILE_HEADER) -> Self {
        Self { 0: raw }
    }
    pub fn number_of_sections(&self) -> usize {
        usize::from(self.0.number_of_sections)
    }
    pub fn arch(&self) -> arch::Architecture {
        arch::Architecture::try_from(self.0.machine).unwrap()
    }
    pub fn size_of_optional_header(&self) -> usize {
        usize::from(self.0.size_of_optional_header)
    }
}
