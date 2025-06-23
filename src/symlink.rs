use std::path::{Path,PathBuf};
use serde::Deserialize;

#[derive(Clone,Debug,Deserialize)]
pub struct Symlink(PathBuf, PathBuf);

impl Symlink {
    pub fn new<P: AsRef<Path>>(src: P, dst: P) -> Symlink
    {
        Symlink(
            src.as_ref().to_path_buf(),
            dst.as_ref().to_path_buf()
            )
    }

    pub fn src(&self) -> &Path {
        &self.0
    }

    pub fn dst(&self) -> &Path {
        &self.1
    }
}
