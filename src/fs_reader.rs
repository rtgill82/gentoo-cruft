extern crate md5;
extern crate threadpool;

use std::collections::HashSet;
use std::{fs,fs::File};
use std::{io,io::Read};
use std::os::unix::fs::FileTypeExt;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::sync::{Arc,Mutex};
use std::time::SystemTime;
use std::mem;

use threadpool::ThreadPool;
use walkdir::DirEntry;
use walkdir::WalkDir;

use crate::file_info::{FileInfo,FileType};
use crate::settings::Settings;

pub struct FsReader {
    pool: ThreadPool,
    results: Arc<Mutex<HashSet<FileInfo>>>,
    settings: Arc<Settings>
}

macro_rules! systime_to_unix {
    ($time:expr) => {
        $time.unwrap()
             .duration_since(SystemTime::UNIX_EPOCH).unwrap()
             .as_secs()
    }
}

impl FsReader {
    pub fn new(settings: Arc<Settings>) -> FsReader {
        let pool = threadpool::Builder::new().build();
        FsReader {
            pool: pool,
            results: Arc::new(Mutex::new(HashSet::new())),
            settings: settings
        }
    }

    pub fn read(&mut self) -> HashSet<FileInfo> {
        let walkdir = WalkDir::new("/");
        let iter = walkdir.into_iter().filter_entry(|entry| {
            Self::is_ignored(self, entry)
        });

        for result in iter {
            match result {
                Ok(entry) => {
                    if let Some(ignore_files) = self.settings.ignore_files() {
                        let pathbuf = entry.path().to_path_buf();
                        if ignore_files.contains(&pathbuf) {
                            continue;
                        }
                    }
                    Self::spawn_stat(self, entry.path().to_path_buf());
                },
                Err(err) => {
                    if self.settings.verbose() {
                        eprintln!("Error accessing path: {}", err);
                    }
                }
            }
        }
        self.pool.join();

        let results = Arc::new(Mutex::new(HashSet::new()));
        let results = mem::replace(&mut self.results, results);
        Arc::try_unwrap(results).unwrap().into_inner().unwrap()
    }

    fn is_ignored(&self, entry: &DirEntry) -> bool {
        let mut rv = true;
        if let Some(ignore_files) = self.settings.ignore_files() {
            if !entry.file_type().is_dir() {
                rv &= !ignore_files.contains(&entry.path().to_path_buf());
            }
        }

        if let Some(ignore_paths) = self.settings.ignore_paths() {
            rv &= !ignore_paths.contains(&entry.path().to_path_buf());
        }
        rv
    }

    fn spawn_stat(&self, entry: PathBuf) {
        let results = self.results.clone();
        let settings = self.settings.clone();
        self.pool.execute(move || {
            let entry = entry;
            match Self::stat(&entry, &settings) {
                Err(err) => {
                    let path = entry.to_string_lossy();
                    if settings.verbose() {
                        eprintln!("Error reading file {}: {}", path, err);
                    }
                },

                Ok(fileinfo) => {
                    let mut set = results.lock().unwrap();
                    set.insert(fileinfo);
                }
            }
        });
    }

    fn stat(filepath: &PathBuf, settings: &Settings)
        -> Result<FileInfo, io::Error>
    {
        let path = String::from(filepath.to_str().unwrap());
        let mut ftype: FileType = FileType::Obj;
        let mut md5: Option<String> = None;
        let mut mtime: Option<u64> = None;
        let mut executable: bool = false;

        let stat = fs::symlink_metadata(&filepath)?;
        if stat.file_type().is_symlink() {
            ftype = FileType::Sym;
            if settings.read_mtime() {
                mtime = Some(systime_to_unix!(stat.modified()));
            }
        } else if stat.file_type().is_dir() {
            ftype = FileType::Dir;
        } else if !stat.file_type().is_fifo()
               && !stat.file_type().is_socket() {
            executable = stat.mode() & 0o111 != 0;

            if settings.read_md5() {
                md5 = Some(calc_md5(&filepath)?);
            }

            if settings.read_mtime() {
                mtime = Some(systime_to_unix!(stat.modified()));
            }
        }

        Ok(FileInfo {
            ftype,
            path,
            md5,
            mtime,
            executable,
            ..Default::default()
        })
    }
}

fn calc_md5(filepath: &PathBuf) -> Result<String, io::Error> {
    let mut context = md5::Context::new();
    let mut file = File::open(filepath)?;
    let mut buf: [u8; 8192] = [0; 8192];

    loop {
        let size = file.read(&mut buf)?;
        if size == 0 { break };
        context.consume(&buf[0..size]);
    }

    let digest = context.compute();
    Ok(format!("{:x}", digest))
}
