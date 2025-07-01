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

use walkdir::{Error,WalkDir};
use crate::package::Package;

pub struct Catalog {
    walkdir: walkdir::IntoIter
}

impl Catalog {
    pub fn new(pkg_dir: &str) -> Catalog {
        let walkdir = WalkDir::new(pkg_dir)
            .max_depth(2)
            .min_depth(2)
            .into_iter();

        Catalog { walkdir }
    }
}

impl Iterator for Catalog {
    type Item = Result<Package, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(result) = self.walkdir.next() {
                match result {
                    Err(err)  => return Some(Err(err)),
                    Ok(entry) => {
                        let mut path = entry.path().to_path_buf();
                        path.push("CONTENTS");
                        if !path.exists() { continue; }
                        let package = Package::new(path);
                        return Some(Ok(package));
                    }
                }
            }
            return None;
        }
    }
}
