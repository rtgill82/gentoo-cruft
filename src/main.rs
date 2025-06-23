#[macro_use]
extern crate clap;
extern crate serde;

use std::collections::HashSet;
use std::process::exit;
use std::sync::Arc;

mod catalog;
mod file_info;
mod fs_reader;
mod package;
mod pkg_reader;
mod settings;
mod symlink;

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

    let settings = Arc::new(settings.unwrap());
    let pkg_reader = PkgReader::new(settings.clone());
    let packages = pkg_reader.read();

    let mut package_files = HashSet::<FileInfo>::new();
    for package in packages {
        for file in package.files() {
            package_files.insert(file.clone());
        }
    }

    let mut fs_reader = FsReader::new(settings.clone());
    let fs_files = fs_reader.read();

    let mut diff = fs_files.difference(&package_files)
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
        let package_files = package_files.iter()
            .map(|item| {
                let mut item = item.clone();
                item.full_hash = true;
                item
            }).collect::<HashSet<FileInfo>>();

        let diff = diff.iter().map(|item| item.clone())
                       .collect::<HashSet<FileInfo>>();

        let diff = fs_files.symmetric_difference(&diff)
            .map(|item| {
                let mut item = item.clone();
                item.full_hash = true;
                item
            }).collect::<HashSet<FileInfo>>();

        let mut diff = diff.difference(&package_files)
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
