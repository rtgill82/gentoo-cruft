extern crate md5;
extern crate threadpool;

use std::collections::HashSet;
use std::{fs,fs::File};
use std::{io,io::Read};
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::sync::{Arc,Mutex};
use std::time::SystemTime;
use std::mem;

use fs_tree::{Error,FsTree,FsTreeBuilder};
use threadpool::ThreadPool;

use crate::file_info::{FileInfo,FileType};
use crate::settings::Settings;

pub struct FsReader<'a> {
    pool: ThreadPool,
    fs_tree: FsTree,
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
    pub fn new(settings: &'a Settings) -> Result<FsReader, Error> {
        let pool = threadpool::Builder::new().build();
        let fs_tree = Self::build_fstree(settings)?;

        Ok(FsReader {
            pool: pool,
            fs_tree: fs_tree,
            results: Arc::new(Mutex::new(HashSet::new())),
            settings: settings
        })
    }

    pub fn read(&mut self) -> HashSet<FileInfo> {
        for result in &self.fs_tree {
            if let Ok(entry) = result {
                let results = self.results.clone();
                let read_md5 = self.settings.read_md5();
                let read_mtime = self.settings.read_mtime();
                self.pool.execute(move || {
                    if let Ok(fileinfo) = Self::stat(entry, read_md5, read_mtime) {
                        let mut set = results.lock().unwrap();
                        set.insert(fileinfo);
                    }
                });
            }
        }
        self.pool.join();

        let results = Arc::new(Mutex::new(HashSet::new()));
        let results = mem::replace(&mut self.results, results);
        Arc::try_unwrap(results).unwrap().into_inner().unwrap()
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

    fn build_fstree(settings: &Settings) -> Result<FsTree, Error> {
        let mut builder = FsTreeBuilder::new("/");
        if let Some(files) = settings.ignore_files() {
            builder.set_ignore_files(files);
        }

        if let Some(paths) = settings.ignore_paths() {
            builder.set_ignore_paths(paths);
        }

        builder.build()
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
