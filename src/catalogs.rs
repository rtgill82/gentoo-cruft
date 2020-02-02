use std::io;
use std::{fs, fs::ReadDir};
use std::path::PathBuf;

pub struct Catalogs {
    top: ReadDir,
    stack: Vec<ReadDir>
}

impl Catalogs {
    pub fn new(pkg_dir: &str) -> Catalogs {
        let stack = Vec::new();
        let top = fs::read_dir(pkg_dir).unwrap();
        Catalogs { top, stack }
    }

    fn stack_next(&mut self) -> Option<io::Result<PathBuf>> {
        let opt = self.stack.last_mut();
        if opt.is_some() {
            let current = opt.unwrap();
            if let Some(result) = current.next() {
                if result.is_err() {
                    return Some(Err(result.unwrap_err()));
                };

                let entry = result.unwrap();
                let mut pathbuf = entry.path();
                pathbuf.push("CONTENTS");
                return Some(Ok(pathbuf));
            }
            self.stack.pop();
        }
        None
    }

    fn top_next(&mut self) -> bool {
        loop {
            let opt = self.top.next();
            if opt.is_none() { return false }

            let result = opt.unwrap();
            if result.is_err() { return true }

            let entry = result.unwrap();
            let pathbuf = entry.path();
            if let Ok(read_dir) = fs::read_dir(pathbuf.clone()) {
                self.stack.push(read_dir);
                return true;
            }
        }
    }
}

impl Iterator for Catalogs {
    type Item = io::Result<PathBuf>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(pathbuf) = Self::stack_next(self) {
                return Some(pathbuf);
            } else {
                if !Self::top_next(self) { return None }
            }
        }
    }
}
