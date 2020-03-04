extern crate serde;

use std::collections::HashSet;
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

    let mut fs_reader = FsReader::new(&settings);
    let files = fs_reader.read();

    let mut diff = files.difference(&catalog)
                        .map(|item| item.clone())
                        .collect::<Vec<FileInfo>>();
    diff.sort_by(|a, b| a.path.cmp(&b.path));

    if !diff.is_empty() {
        println!("Files not in package database:");
        for file in &diff {
            println!("  {}", file);
        }
    }

    if settings.read_md5() || settings.read_mtime() {
        let catalog = catalog.iter()
                             .map(|item| {
                                 let mut item = item.clone();
                                 item.full_hash = true;
                                 item
                             }).collect::<HashSet<FileInfo>>();

        let diff = diff.iter().map(|item| item.clone())
                       .collect::<HashSet<FileInfo>>();

        let diff = files.symmetric_difference(&diff)
                        .map(|item| {
                            let mut item = item.clone();
                            item.full_hash = true;
                            item
                        }).collect::<HashSet<FileInfo>>();

        let mut diff = diff.difference(&catalog)
                       .map(|item| item.clone())
                       .collect::<Vec<FileInfo>>();
        diff.sort_by(|a, b| a.path.cmp(&b.path));

        if !diff.is_empty() {
            println!("\nFiles that have been modified:");
            for file in &diff {
                println!("  {}", file);
            }
        }
    }
}
