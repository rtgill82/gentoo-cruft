extern crate fs_tree;
extern crate serde;

use std::process::exit;

mod catalogs;
mod file_info;
mod fs_reader;
mod pkg_reader;
mod settings;

use file_info::FileInfo;
use fs_reader::FsReader;
use pkg_reader::PkgReader;
use settings::Settings;

fn main() {
    let settings = Settings::new();
    if let Err(e) = settings {
        println!("Error reading configuration: {}", e);
        exit(1);
    }

    let settings = settings.unwrap();
    let pkg_reader = PkgReader::new(&settings);
    let catalog = pkg_reader.read();

    let mut fs_reader = FsReader::new(&settings).unwrap();
    let files = fs_reader.read();

    let mut cruft: Vec<&FileInfo> = files.difference(&catalog).collect();
    cruft.sort_by(|a, b| a.path.cmp(&b.path));
    for file in cruft {
        println!("  {}", file);
    }
}
