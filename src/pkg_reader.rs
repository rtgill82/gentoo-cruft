extern crate md5;
extern crate threadpool;

use std::fs::File;
use std::io::{BufRead,BufReader};
use std::path::PathBuf;
use std::sync::{Arc,Mutex};

use threadpool::ThreadPool;

use crate::catalog::Catalog;
use crate::package::Package;
use crate::settings::Settings;
use crate::file_info::{FileInfo,FileType};

pub struct PkgReader {
    pool: ThreadPool,
    settings: Arc<Settings>
}

impl PkgReader {
    pub fn new(settings: Arc<Settings>) -> PkgReader {
        let pool = threadpool::Builder::new().build();
        PkgReader { pool, settings }
    }

    pub fn read(&self) -> Vec<Package> {
        let vec = Arc::new(Mutex::new(Vec::new()));
        for package in Catalog::new(self.settings.pkg_dir()).unwrap() {
            if let Ok(package) = package {
                let vec = vec.clone();
                let settings = self.settings.clone();
                self.pool.execute(move || {
                    let package = Self::read_package(package, &settings);
                    let mut vec = vec.lock().unwrap();
                    vec.push(package);
                });
            }
        }
        self.pool.join();
        Arc::try_unwrap(vec).unwrap().into_inner().unwrap()
    }

    fn read_package(mut package: Package, settings: &Settings) -> Package {
        let file = File::open(package.contents_path()).unwrap();
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.unwrap();
            Self::parse_entry(&mut package, &line, settings);
        }
        package
    }

    fn parse_entry(package: &mut Package, s: &str, settings: &Settings) {
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

                if settings.read_md5() {
                    md5 = Some(String::from(fields[len - 2]));
                }

                if settings.read_mtime() {
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
                if settings.read_mtime() {
                    mtime = Some(fields[len - 1].parse().unwrap());
                }
            }
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

        let file_info = FileInfo {
            ftype,
            path,
            md5,
            mtime,
            ..Default::default()
        };

        package.add_file(file_info);
    }
}
