extern crate md5;
extern crate threadpool;

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead,BufReader};
use std::sync::{Arc,Mutex};

use threadpool::ThreadPool;

use crate::catalogs::Catalogs;
use crate::file_info::{FileInfo,FileType};
use crate::settings::Settings;

pub struct PkgReader<'a> {
    pool: ThreadPool,
    settings: &'a Settings
}

impl<'a> PkgReader<'a> {
    pub fn new(settings: &'a Settings) -> PkgReader {
        let pool = threadpool::Builder::new().build();
        PkgReader { pool, settings }
    }

    pub fn read(&self) -> HashSet<FileInfo> {
        let set = Arc::new(Mutex::new(HashSet::new()));
        for catalog in Catalogs::new(self.settings.pkg_dir()) {
            if let Ok(pathbuf) = catalog {
                let set = set.clone();
                let read_md5 = self.settings.read_md5();
                let read_mtime = self.settings.read_mtime();
                self.pool.execute(move || {
                    let file = File::open(pathbuf).unwrap();
                    let reader = BufReader::new(file);
                    for line in reader.lines() {
                        let line = line.unwrap();
                        let mut set = set.lock().unwrap();
                        set.insert(Self::parse_entry(&line, read_md5, read_mtime));
                    }
                });
            }
        }
        self.pool.join();
        Arc::try_unwrap(set).unwrap().into_inner().unwrap()
    }

    fn parse_entry(s: &str, read_md5: bool, read_mtime: bool) -> FileInfo {
        let ftype: FileType;
        let path: String;
        let mut md5: Option<String> = None;
        let mut mtime: Option<u64> = None;

        let fields = s.split(' ').collect::<Vec<_>>();
        ftype = FileType::from(fields[0]);
        match ftype {
            FileType::Obj => {
                let len = fields.len();
                path = fields[1..=(len - 3)].join(" ");

                if read_md5 {
                    md5 = Some(String::from(fields[len - 2]));
                }

                if read_mtime {
                    mtime = Some(fields[len - 1].parse().unwrap());
                }
            },

            FileType::Dir => {
                let len = fields.len();
                path = fields[1..=(len - 1)].join(" ");
            },

            FileType::Sym => {
                let split = s.split(" -> ")
                             .collect::<Vec<_>>()[0]
                             .split(" ").collect::<Vec<_>>();

                let len = split.len();
                path = split[1..=(len - 1)].join(" ");

                let len = fields.len();
                if read_mtime {
                    mtime = Some(fields[len - 1].parse().unwrap());
                }
            }
        };

        FileInfo { ftype, path, md5, mtime, executable: false }
    }
}
