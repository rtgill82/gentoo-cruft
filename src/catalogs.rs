use std::path::PathBuf;

use walkdir::Error;
use walkdir::WalkDir;

pub struct Catalogs {
    walkdir: walkdir::IntoIter
}

impl Catalogs {
    pub fn new(pkg_dir: &str) -> Result<Catalogs, Error> {
        let walkdir = WalkDir::new(pkg_dir)
            .max_depth(2)
            .min_depth(2)
            .into_iter();

        Ok(Catalogs { walkdir })
    }
}

impl Iterator for Catalogs {
    type Item = Result<PathBuf, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(result) = self.walkdir.next() {
                match result {
                    Err(err)  => return Some(Err(err)),
                    Ok(entry) => {
                        let mut path = entry.path().to_path_buf();
                        path.push("CONTENTS");
                        if !path.exists() { continue; }
                        return Some(Ok(path));
                    }
                }
            }
            return None;
        }
    }
}
