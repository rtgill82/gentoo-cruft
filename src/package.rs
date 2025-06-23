use std::path::{Path,PathBuf};
use crate::FileInfo;

#[derive(Debug)]
pub struct Package {
    contents_path: PathBuf,
    files: Vec<FileInfo>,
}

impl Package {
    pub fn new(contents_path: PathBuf) -> Package {
        Package {
            contents_path: contents_path,
            files: Vec::new()
        }
    }

    pub fn contents_path(&self) -> &Path {
        &self.contents_path
    }

    pub fn files(&self) -> &[FileInfo] {
        &self.files
    }

    pub fn add_file(&mut self, file_info: FileInfo) {
        self.files.push(file_info);
    }
}
