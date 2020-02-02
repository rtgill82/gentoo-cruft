extern crate md5;
extern crate threadpool;

use std::collections::HashSet;
use std::{fs,fs::File};
use std::{io,io::Read};
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{Arc,Mutex};
use std::time::SystemTime;
use std::mem;

use threadpool::ThreadPool;

use crate::file_info::{FileInfo,FileType};
use crate::settings::Settings;

pub struct FsReader<'a> {
    pool: ThreadPool,
    results: Arc<Mutex<HashSet<FileInfo>>>,
    settings: &'a Settings
}

macro_rules! systime_to_unix {
    ($time:expr) => {
        $time.unwrap()
             .duration_since(SystemTime::UNIX_EPOCH).unwrap()
             .as_secs()
    }
}

impl<'a> FsReader<'a> {
    pub fn new(settings: &'a Settings) -> FsReader {
        let pool = threadpool::Builder::new().build();
        FsReader {
            pool: pool,
            results: Arc::new(Mutex::new(HashSet::new())),
            settings: settings
        }
    }

    pub fn stat(filepath: PathBuf, read_md5: bool, read_mtime: bool) -> Result<FileInfo, io::Error> {
        let ftype: FileType;
        let path: String;
        let mut md5: Option<String> = None;
        let mut mtime: Option<u64> = None;
        let mut executable: bool = false;

        let stat = fs::symlink_metadata(&filepath)?;
        if stat.file_type().is_symlink() {
            ftype = FileType::Sym;
            path = String::from(filepath.to_str().unwrap());

            if read_mtime {
                mtime = Some(systime_to_unix!(stat.modified()));
            }
        } else if stat.file_type().is_dir() {
            ftype = FileType::Dir;
            path = String::from(filepath.to_str().unwrap());
        } else {
            ftype = FileType::Obj;
            path = String::from(filepath.to_str().unwrap());
            executable = stat.mode() & 0o111 != 0;

            if read_md5 {
                md5 = Some(calc_md5(&filepath)?);
            }

            if read_mtime {
                mtime = Some(systime_to_unix!(stat.modified()));
            }
        }

        Ok(FileInfo { ftype, path, md5, mtime, executable })
    }

    pub fn read(&mut self) -> HashSet<FileInfo> {
        Self::read_dir(Pin::new(self), PathBuf::from("/")).unwrap();
        self.pool.join();

        let results = Arc::new(Mutex::new(HashSet::new()));
        let results = mem::replace(&mut self.results, results);
        Arc::try_unwrap(results).unwrap().into_inner().unwrap()
    }

    fn read_dir<'r>(self: Pin<&'r Self>, path: PathBuf) -> Result<(), io::Error> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if Self::ignore_path(&self, &entry.path()) { continue; }

            if !Self::ignore_file(&self, &entry.path()) {
                let pathbuf = entry.path();
                let results = self.results.clone();
                let read_md5 = self.settings.read_md5();
                let read_mtime = self.settings.read_mtime();
                self.pool.execute(move || {
                    if let Ok(fileinfo) = FsReader::stat(pathbuf, read_md5, read_mtime) {
                        let mut set = results.lock().unwrap();
                        set.insert(fileinfo);
                    }
                });
            }

            let stat = fs::symlink_metadata(&entry.path())?;
            if !stat.file_type().is_symlink() && stat.is_dir() {
                let _ = Self::read_dir(self, entry.path());
            }
        }
        Ok(())
    }

    fn ignore_file(&self, path: &PathBuf) -> bool {
        if let Some(ignore_files) = self.settings.ignore_files() {
            if ignore_files.contains(path) { return true; }
        }
        false
    }

    fn ignore_path(&self, path: &PathBuf) -> bool {
        if let Some(ignore_paths) = self.settings.ignore_paths() {
            if ignore_paths.contains(path) { return true; }
        }
        false
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
