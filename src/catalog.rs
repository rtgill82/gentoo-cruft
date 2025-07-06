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

pub mod file;
mod package;

use std::sync::{Arc,Mutex};
use walkdir::WalkDir;

use crate::Settings;
use self::package::Package;
pub use self::file::File;

pub struct Catalog;

impl Catalog {
    pub fn read() -> Vec<File> {
        let settings = Settings::get();
        let pool = threadpool::Builder::new().build();
        let walkdir = WalkDir::new(settings.pkg_dir())
            .max_depth(2)
            .min_depth(2)
            .into_iter();

        let vec = Arc::new(Mutex::new(Vec::new()));
        for result in walkdir {
            match result {
                Ok(entry) => {
                    let mut path = entry.path().to_path_buf();
                    path.push("CONTENTS");
                    if !path.exists() { continue; }
                    
                    let vec = vec.clone();
                    let settings = Settings::get();
                    pool.execute(move || {
                        let mut vec = vec.lock().unwrap();
                        vec.append(&mut Package::read(path, &settings));
                    });
                },

                Err(err) => {
                    if settings.verbose() {
                        eprintln!("Error traversing directory tree: {}", err);
                    }
                }
            }
        }

        pool.join();
        Arc::try_unwrap(vec).unwrap().into_inner().unwrap()
    }
}
