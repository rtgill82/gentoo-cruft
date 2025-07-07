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

#[macro_use]
extern crate clap;
extern crate serde;

use std::any::Any;
use std::borrow::Borrow;
use std::collections::HashSet;

mod catalog;
mod file_info;
mod file_system;
mod settings;
mod symlink;

use catalog::Catalog;
use file_info::FileInfo;
use file_system::{File,FileSystem};
use settings::Settings;

fn main() {
    let pkg_files: HashSet<Box<dyn FileInfo>> = Catalog::read()
        .into_iter()
        .map(|file| file.to_file_info())
        .collect();

    let fs_files:HashSet<Box<dyn FileInfo>> = FileSystem::read()
        .into_iter()
        .map(|file| file.to_file_info())
        .collect();


    let settings = Settings::get();
    let mut diff: HashSet<_> = fs_files.difference(&pkg_files).collect();
    if settings.md5() || settings.mtime() {
        let modified = find_modified_files(&pkg_files, &fs_files);
        diff.extend(modified);
    }

    let mut diff: Vec<File> = diff.iter().map(|file| {
        let file: &dyn Any = file.as_ref();
        match file.downcast_ref::<File>() {
            Some(file) => file.clone(),
            None => panic!("Unable to downcast File!")
        }
    }).collect();

    diff.sort_by(|a, b| a.path().cmp(b.path()));
    for file in diff {
        println!("{file}");
    }
}

fn find_modified_files<'a>(pkg_files: &'a HashSet<Box<dyn FileInfo>>,
                           fs_files:  &'a HashSet<Box<dyn FileInfo>>)
    -> HashSet<&'a Box<dyn FileInfo>>
{
    let settings = Settings::get();

    let xset: HashSet<_> = pkg_files.iter()
        .filter_map(|pkg_file| {
            if fs_files.contains(pkg_file) {
                return Some(pkg_file);
            }

            None
        }).collect();

    fs_files.into_iter().filter_map(|fs_file| {
        if let Some(xset_file) = xset.get(fs_file) {
            let xset_file: &dyn FileInfo = (**xset_file).borrow();
            if settings.md5() && !fs_file.md5_matches(xset_file)
            {
                return Some(fs_file);
            }

            if settings.mtime() && !fs_file.mtime_matches(xset_file)
            {
                return Some(fs_file);
            }
        }

        None
    }).collect()
}
