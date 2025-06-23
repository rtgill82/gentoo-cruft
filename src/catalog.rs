use walkdir::{Error,WalkDir};
use crate::package::Package;

pub struct Catalog {
    walkdir: walkdir::IntoIter
}

impl Catalog {
    pub fn new(pkg_dir: &str) -> Result<Catalog, Error> {
        let walkdir = WalkDir::new(pkg_dir)
            .max_depth(2)
            .min_depth(2)
            .into_iter();

        Ok(Catalog { walkdir })
    }
}

impl Iterator for Catalog {
    type Item = Result<Package, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(result) = self.walkdir.next() {
                match result {
                    Err(err)  => return Some(Err(err)),
                    Ok(entry) => {
                        let mut path = entry.path().to_path_buf();
                        path.push("CONTENTS");
                        if !path.exists() { continue; }
                        let package = Package::new(path);
                        return Some(Ok(package));
                    }
                }
            }
            return None;
        }
    }
}
