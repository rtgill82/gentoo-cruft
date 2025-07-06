//
// Copyright (C) 2025 Robert Gill <rtgill82@gmail.com>
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

use std::path::{Path,PathBuf};
use crate::file_info::FileInfo;

#[derive(Debug)]
pub struct File {
    path: PathBuf,
    file_type: FileType,
    mtime: u64,
    md5: Option<String>
}

#[derive(Clone,Debug,Hash,PartialEq)]
pub enum FileType {
    Dir,
    Obj,
    Sym(PathBuf)
}

impl File {
    pub fn new<P>(path: P, file_type: FileType, md5: String, mtime: u64)
        -> File where P: AsRef<Path>
    {
        let md5 = if md5.is_empty() {
            None
        } else {
            Some(md5)
        };

        File {
            path: path.as_ref().to_path_buf(),
            file_type: file_type,
            mtime: mtime,
            md5: md5
        }
    }

    pub fn to_file_info(self) -> Box<dyn FileInfo> {
        Box::new(self)
    }
}

impl FileInfo for File {
    fn path(&self) -> &Path {
        &self.path
    }

    fn file_type(&self) -> FileType {
        self.file_type.clone()
    }

    fn mtime(&self) -> u64 {
        self.mtime
    }

    fn md5(&self) -> Option<&str> {
        self.md5.as_deref()
    }

    fn md5_matches(&self, _value: bool) { }

    fn mtime_matches(&self, _value: bool) { }
}
