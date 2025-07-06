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

use std::io::{BufRead,BufReader};
use std::io::Lines;
use std::path::{Path,PathBuf};
use std::fs;

use crate::Settings;
use super::file::{File,FileType};

pub struct Package<'a> {
    lines: Lines<BufReader<fs::File>>,
    settings: &'a Settings
}

impl<'a> Package<'a> {
    pub fn read<P>(contents_path: P, settings: &'a Settings) -> Vec<File>
        where P: AsRef<Path>
    {
        let file = fs::File::open(contents_path.as_ref()).unwrap();
        let reader = BufReader::new(file);
        let lines = reader.lines();

        Package { lines, settings }.collect()
    }
}

impl Iterator for Package<'_> {
    type Item = File;

    fn next(&mut self) -> Option<Self::Item> {
        match self.lines.next() {
            Some(line) => Some(parse_entry(&line.unwrap(), &self.settings)),
            None => None
        }
    }
}

fn parse_entry(line: &str, settings: &Settings) -> File {
    let path: String;
    let file_type: FileType;
    let md5: String;
    let mtime: u64;

    let fields = line.split(' ').collect::<Vec<_>>();
    match fields[0] {
        "obj" => {
            let len = fields.len();
            path = fields[1..=(len - 3)].join(" ");
            file_type = FileType::Obj;
            md5 = String::from(fields[len - 2]);
            mtime = fields[len - 1].parse().unwrap();
        },

        "dir" => {
            let len = fields.len();
            path = fields[1..=(len - 1)].join(" ");
            file_type = FileType::Dir;
            md5 = String::from("");
            mtime = 0;
        },

        "sym" => {
            let split = line.split(" -> ").collect::<Vec<_>>();
            let len = split[0].len();
            path = split[0][4..=(len - 1)].to_string();
            let split = split[1].split(" ").collect::<Vec<_>>();
            let len = split.len();
            let target = split[0..=(len - 1)][0].to_string();
            file_type = FileType::Sym(PathBuf::from(target));
            md5 = String::from("");
            let len = fields.len();
            mtime = fields[len - 1].parse().unwrap();
        },

        _ => panic!("Unrecognized file type!")
    };

    let mut path = PathBuf::from(path);
    if !settings.is_split_usr() {
        if let Some(links) = settings.links_to_usr() {
            for link in links {
                if path.starts_with(link.src()) {
                    let count = link.src().components().count();
                    let pathbuf: PathBuf = path.components().skip(count).collect();
                    path = PathBuf::from(link.dst()).join(pathbuf);
                }
            }
        }
    }

    File::new(path, file_type, md5, mtime)
}
