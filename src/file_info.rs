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

use std::any::Any;
use std::hash::{Hash,Hasher};
use std::path::Path;

use crate::Settings;
use crate::catalog::file::FileType;

pub trait FileInfo: Any {
    fn path(&self) -> &Path;
    fn file_type(&self) -> FileType;
    fn mtime(&self) -> u64;
    fn md5(&self) -> Option<&str>;
    fn md5_matches(&self, value: bool);
    fn mtime_matches(&self, value: bool);
}

impl Eq for dyn FileInfo { }

impl PartialEq for dyn FileInfo {
    fn eq(&self, other: &dyn FileInfo) -> bool {
        let mut rv = self.path() == other.path() &&
            self.file_type() == other.file_type();

        let settings = Settings::get();
        if settings.md5() && rv == true {
            rv &= self.md5() == other.md5();
            self.md5_matches(rv);
            other.md5_matches(rv);
        }

        if settings.mtime() && rv == true {
            rv &= self.mtime() == other.mtime();
            self.mtime_matches(rv);
            other.mtime_matches(rv);
        }

        rv
    }
}

impl Hash for dyn FileInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path().hash(state);
        self.file_type().hash(state);

        let settings = Settings::get();
        if settings.mtime() {
            self.mtime().hash(state);
        }

        if settings.md5() {
            self.md5().hash(state);
        }
    }
}
