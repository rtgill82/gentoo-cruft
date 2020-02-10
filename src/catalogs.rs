use fs_tree::Error;
use fs_tree::FsTreeBuilder;
use fs_tree::IntoIter;
use fs_tree::Result as FsTreeResult;

pub struct Catalogs {
    fs_tree: IntoIter
}

impl Catalogs {
    pub fn new(pkg_dir: &str) -> Result<Catalogs, Error> {
        let fs_tree = FsTreeBuilder::new(pkg_dir)
            .max_depth(2)
            .min_depth(2).build()?
            .into_iter();
        Ok(Catalogs { fs_tree })
    }
}

impl Iterator for Catalogs {
    type Item = FsTreeResult;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(result) = self.fs_tree.next() {
                match result {
                    Err(err)  => return Some(Err(err)),
                    Ok(mut entry) => {
                        entry.push("CONTENTS");
                        if !entry.exists() { continue; }
                        return Some(Ok(entry));
                    }
                }
            }
            return None;
        }
    }
}
