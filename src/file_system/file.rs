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

use std::cell::RefCell;
use std::io::Read;
use std::path::{Path,PathBuf};
use std::{fmt,fs,io};
use md5;

use crate::catalog::file::FileType;
use crate::file_info::FileInfo;

#[derive(Debug)]
pub struct File {
    path: PathBuf,
    stat: RefCell<Stat>,
    mtime: u64,
    md5: Option<String>
}

#[derive(Clone,Debug,Hash)]
pub enum Stat {
    Directory,
    Regular,
    Executable,
    Suid,
    Symlink(PathBuf),
    BlockDevice,
    CharDevice,
    Fifo,
    Socket,
    MD5,
    MTIME
}

impl File {
    pub fn new<P>(path: P, stat: Stat, mtime: u64) -> File
        where P: AsRef<Path>
    {
        File {
            path: path.as_ref().to_path_buf(),
            stat: RefCell::new(stat),
            mtime: mtime,
            md5: None
        }
    }

    pub fn calc_md5(mut self) -> io::Result<Self> {
        let mut context = md5::Context::new();
        let mut file = fs::File::open(&self.path)?;
        let mut buf: [u8; 8192] = [0; 8192];

        loop {
            let size = file.read(&mut buf)?;
            if size == 0 { break };
            context.consume(&buf[0..size]);
        }

        let digest = context.finalize();
        self.md5 = Some(format!("{:x}", digest));

        Ok(self)
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
        let stat = self.stat.borrow();
        match &*stat {
            Stat::Directory => FileType::Dir,
            Stat::Regular => FileType::Obj,
            Stat::Executable => FileType::Obj,
            Stat::Suid => FileType::Obj,
            Stat::Symlink(link) => FileType::Sym(link.clone()),
            Stat::BlockDevice => FileType::Obj,
            Stat::CharDevice => FileType::Obj,
            Stat::Fifo => FileType::Obj,
            Stat::Socket => FileType::Obj,
            _ => panic!("Invalid file type: {:?}", stat)
        }
    }

    fn mtime(&self) -> u64 {
        self.mtime
    }

    fn md5(&self) -> Option<&str> {
        self.md5.as_deref()
    }

    fn md5_matches(&self, value: bool) {
        if !value {
            self.stat.replace(Stat::MD5);
        }
    }

    fn mtime_matches(&self, value: bool) {
        if !value {
            self.stat.replace(Stat::MTIME);
        }
    }
}

impl fmt::Display for Stat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ch: char = match self {
            Stat::Directory => 'D',
            Stat::Regular => 'R',
            Stat::Executable => 'E',
            Stat::Suid => 'S',
            Stat::Symlink(_) => 'L',
            Stat::BlockDevice => 'B',
            Stat::CharDevice => 'C',
            Stat::Fifo => 'F',
            Stat::Socket => 'Z',
            Stat::MD5 => 'M',
            Stat::MTIME => 'T'
        };

        write!(f, "{}", ch)
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.stat.borrow(), self.path.to_string_lossy())
    }
}
