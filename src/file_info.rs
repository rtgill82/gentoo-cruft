//
// Copyright (C) 2020,2025 Robert Gill <rtgill82@gmail.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to
// deal in the Software without restriction, including without limitation the
// rights to use, copy, modify, merge, publish, distribute, sublicense, and/or
// sell copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies of the Software, its documentation and marketing & publicity
// materials, and acknowledgment shall be given in the documentation, materials
// and software packages that this Software was used.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
// THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
// IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
// CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
//

use std::cmp::PartialEq;
use std::{fmt,fmt::Display};
use std::hash::{Hash,Hasher};
use std::path::PathBuf;

#[derive(Clone,Debug,Default,Eq)]
pub struct FileInfo {
    pub ftype: FileType,
    pub path: PathBuf,
    pub md5: Option<String>,
    pub mtime: Option<u64>,
    pub executable: bool,
    pub full_hash: bool
}

impl Display for FileInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let exec = if self.executable { "*" } else { "" };
        write!(f, "{}{}", exec, self.path.to_string_lossy())
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
