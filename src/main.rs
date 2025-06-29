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

use std::collections::HashSet;
use std::process::exit;
use std::sync::Arc;

mod catalog;
mod file_info;
mod fs_reader;
mod package;
mod pkg_reader;
mod settings;
mod symlink;

use file_info::FileInfo;
use fs_reader::FsReader;
use pkg_reader::PkgReader;
use settings::Settings;

fn main() {
    let settings = Settings::new();
    if let Err(e) = settings {
        println!("Error reading configuration: {}", e);
        exit(1);
    }

    let settings = Arc::new(settings.unwrap());
    let pkg_reader = PkgReader::new(settings.clone());
    let packages = pkg_reader.read();

    let mut package_files = HashSet::<FileInfo>::new();
    for package in packages {
        for file in package.files() {
            package_files.insert(file.clone());
        }
    }

    let mut fs_reader = FsReader::new(settings.clone());
    let fs_files = fs_reader.read();

    let mut diff = fs_files.difference(&package_files)
                        .map(|item| item.clone())
                        .collect::<Vec<FileInfo>>();
    diff.sort_by(|a, b| a.path.cmp(&b.path));

    if !diff.is_empty() {
        println!("Files not in package database:");
        for file in &diff {
            println!("  {}", file);
        }
    }

    if settings.read_md5() || settings.read_mtime() {
        let package_files = package_files.iter()
            .map(|item| {
                let mut item = item.clone();
                item.full_hash = true;
                item
            }).collect::<HashSet<FileInfo>>();

        let diff = diff.iter().map(|item| item.clone())
                       .collect::<HashSet<FileInfo>>();

        let diff = fs_files.symmetric_difference(&diff)
            .map(|item| {
                let mut item = item.clone();
                item.full_hash = true;
                item
            }).collect::<HashSet<FileInfo>>();

        let mut diff = diff.difference(&package_files)
                       .map(|item| item.clone())
                       .collect::<Vec<FileInfo>>();
        diff.sort_by(|a, b| a.path.cmp(&b.path));

        if !diff.is_empty() {
            println!("\nFiles that have been modified:");
            for file in &diff {
                println!("  {}", file);
            }
        }
    }
}
