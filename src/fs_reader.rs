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

pub struct FsReader {
    ignore_paths: Option<Vec<PathBuf>>,
    pool: ThreadPool,
    results: Arc<Mutex<HashSet<FileInfo>>>,
    read_md5: bool,
    read_mtime: bool
}

macro_rules! systime_to_unix {
    ($time:expr) => {
        $time.unwrap()
             .duration_since(SystemTime::UNIX_EPOCH).unwrap()
             .as_secs()
    }
}

impl FsReader {
    pub fn new(ignore_paths: Option<&[&str]>, read_md5: bool, read_mtime: bool) -> FsReader {
        let mut paths: Option<Vec<PathBuf>> = None;
        if let Some(ignore_paths) = ignore_paths {
            let mut vec = Vec::new();
            for path in ignore_paths {
                vec.push(PathBuf::from(*path))
            }
            paths = Some(vec)
        }

        let pool = threadpool::Builder::new().build();
        FsReader {
            ignore_paths: paths,
            pool: pool,
            results: Arc::new(Mutex::new(HashSet::new())),
            read_md5: read_md5,
            read_mtime: read_mtime
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

    fn read_dir<'a>(self: Pin<&'a Self>, path: PathBuf) -> Result<(), io::Error> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if let Some(ignore_paths) = &self.ignore_paths {
                if ignore_paths.contains(&entry.path()) {
                    continue;
                }
            }

            let pathbuf = entry.path();
            let results = self.results.clone();
            let read_md5 = self.read_md5;
            let read_mtime = self.read_mtime;
            self.pool.execute(move || {
                if let Ok(fileinfo) = FsReader::stat(pathbuf, read_md5, read_mtime) {
                    let mut set = results.lock().unwrap();
                    set.insert(fileinfo);
                }
            });

            let stat = fs::symlink_metadata(&entry.path())?;
            if !stat.file_type().is_symlink() && stat.is_dir() {
                let _ = Self::read_dir(self, entry.path());
            }
        }
        Ok(())
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
