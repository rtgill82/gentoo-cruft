extern crate clap;

mod catalogs;
mod file_info;
mod fs_reader;
mod pkg_reader;

use pkg_reader::PkgReader;
use fs_reader::FsReader;

const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHOR: &'static str = env!("CARGO_PKG_AUTHORS");

static PKG_DIR: &str = "/var/db/pkg";
static IGNORE_PATHS: &[&str] = &[
    "/boot",
    "/dev",
    "/etc/ssl/certs",
    "/home",
    "/proc",
    "/root",
    "/run",
    "/sys",
    "/usr/src",
    "/var/cache/distfiles",
    "/var/db/pkg",
    "/var/db/repos/gentoo"
];

fn main() {
    let args = parse_args();
    let read_md5 = args.is_present("md5");
    let read_mtime = args.is_present("mtime");

    let pkg_reader = PkgReader::new(read_md5, read_mtime);
    let catalog = pkg_reader.read();

    let mut fs_reader = FsReader::new(Some(IGNORE_PATHS), read_md5, read_mtime);
    let files = fs_reader.read();

    let cruft = files.difference(&catalog);
    for file in cruft {
        println!("  {}", file);
    }
}

fn parse_args() -> clap::ArgMatches<'static> {
    clap::App::new(NAME)
        .version(VERSION)
        .author(AUTHOR)
        .arg(clap::Arg::with_name("md5")
             .long("md5")
             .short("m")
             .help("Calculate and compare MD5 sums"))
        .arg(clap::Arg::with_name("mtime")
             .long("mtime")
             .short("t")
             .help("Compare file modification times"))
        .get_matches()
}
