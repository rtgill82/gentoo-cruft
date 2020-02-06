use std::cmp::PartialEq;
use std::{fmt,fmt::Display};
use std::hash::{Hash,Hasher};

#[derive(Clone,Debug,Default,Eq)]
pub struct FileInfo {
    pub ftype: FileType,
    pub path: String,
    pub md5: Option<String>,
    pub mtime: Option<u64>,
    pub executable: bool,
    pub full_hash: bool
}

impl Display for FileInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let exec = if self.executable { "*" } else { "" };
        write!(f, "{}{}", exec, self.path)
    }
}

impl Hash for FileInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ftype.hash(state);
        self.path.hash(state);

        if self.full_hash {
            self.md5.hash(state);
            self.mtime.hash(state);
        }
    }
}

impl PartialEq for FileInfo {
    fn eq(&self, other: &Self) -> bool {
        let mut rv = self.ftype == other.ftype &&
            self.path == other.path;

        if self.full_hash {
            rv = rv && self.md5 == other.md5 &&
                self.mtime == other.mtime
        }

        rv
    }
}

#[derive(Clone,Debug,Eq,Hash,PartialEq)]
pub enum FileType { Dir, Obj, Sym }

impl Default for FileType {
    fn default() -> Self { FileType::Obj }
}

impl From<&str> for FileType {
    fn from(s: &str) -> FileType {
        match s {
            "obj" => FileType::Obj,
            "dir" => FileType::Dir,
            "sym" => FileType::Sym,
            _     => panic!("Unexpected file type")
        }
    }
}
