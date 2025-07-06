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

pub mod file;

use std::os::unix::fs::FileTypeExt;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::sync::{Arc,Mutex};
use std::time::SystemTime;
use std::{fs,io};

use walkdir::{DirEntry,WalkDir};

use crate::Settings;
use self::file::Stat;
pub use self::file::File;

macro_rules! systime_to_unix {
    ($time:expr) => {
        $time.unwrap()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("File modification time before UNIX EPOCH!")
            .as_secs()
    }
}

pub struct FileSystem;

impl FileSystem {
    pub fn read() -> Vec<File> {
        let settings = Settings::get();
        let walkdir = WalkDir::new("/");
        let walkdir = walkdir.into_iter()
            .filter_entry(|e| {
                is_ignored(e, &settings)
            });

        let vec = Arc::new(Mutex::new(Vec::new()));
        let pool = threadpool::Builder::new().build();
        for result in walkdir {
            match result {
                Ok(entry) => {
                    let path = entry.path().to_path_buf();
                    let settings = Settings::get();
                    let vec = vec.clone();

                    pool.execute(move || {
                        match stat(path, settings) {
                            Ok(file) => {
                                let mut vec = vec.lock().unwrap();
                                vec.push(file);
                            },

                            Err(_) => { }
                        }
                    });
                },

                Err(err) => {
                    if settings.verbose() {
                        eprintln!("Error accessing path: {}", err);
                    }
                }
            }
        }

        pool.join();
        Arc::try_unwrap(vec).unwrap().into_inner().unwrap()
    }
}

fn is_ignored(entry: &DirEntry, settings: &Settings) -> bool {
    let mut rv = true;
    if !entry.file_type().is_dir() {
        if let Some(ignore_files) = settings.ignore_files() {
            rv &= !ignore_files.iter().any(|e| e == entry.path());
        }
    } else {
        if let Some(ignore_paths) = settings.ignore_paths() {
            rv &= !ignore_paths.iter().any(|e| e == entry.path());
        }
    }

    rv
}

fn stat(path: PathBuf, settings: Arc<Settings>) -> io::Result<File>
{
    let stat: Stat;
    let mtime: u64;

    let metadata = fs::symlink_metadata(&path)?;
    mtime = systime_to_unix!(metadata.modified());

    if metadata.file_type().is_symlink() {
        let target = fs::read_link(&path)?;
        stat = Stat::Symlink(target);
    } else if metadata.file_type().is_dir() {
        stat = Stat::Directory;
    } else if metadata.file_type().is_block_device() {
        stat = Stat::BlockDevice;
    } else if metadata.file_type().is_char_device() {
        stat = Stat::CharDevice;
    } else if metadata.file_type().is_fifo() {
        stat = Stat::Fifo;
    } else if metadata.file_type().is_socket() {
        stat = Stat::Socket;
    } else {
        if metadata.mode() & 0o4_000 != 0 {
            stat = Stat::Suid;
        } else if metadata.mode() & 0o111 != 0 {
            stat = Stat::Executable;
        } else {
            stat = Stat::Regular;
        }
    }

    let file = File::new(path, stat, mtime);
    if settings.md5() {
        file.calc_md5()
    } else {
        Ok(file)
    }
}
