extern crate clap;
extern crate config;

use std::collections::HashSet;
use std::path::PathBuf;

use clap::ArgMatches;
use config::{Config,ConfigError,File};
use serde::Deserialize;

const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHOR: &'static str = env!("CARGO_PKG_AUTHORS");

macro_rules! unwrap_array_arg {
    ($e:expr) => {
        $e.unwrap().map(|s| PathBuf::from(s)).collect::<HashSet<PathBuf>>()
    }
}

macro_rules! unwrap_bool_arg {
    ($e:expr) => {
        match $e.unwrap_or("n") {
            "y" => true,
            "n" => false,
            _   => false
        }
    }
}

#[derive(Debug,Default,Deserialize)]
pub struct Settings {
    pkg_dir: String,
    ignore_files: Option<HashSet<PathBuf>>,
    ignore_paths: Option<HashSet<PathBuf>>,
    md5: bool,
    mtime: bool
}

impl Settings {
    pub fn new() -> Result<Self,ConfigError> {
        let args = parse_args();
        let mut conf = Config::new();

        conf.set_default("md5", false)?;
        conf.set_default("mtime", false)?;
        conf.set_default::<Option<Vec<String>>>("ignore_paths", None)?;
        conf.set_default::<Option<Vec<String>>>("ignore_files", None)?;
        conf.merge(File::with_name("config/cruft.yaml"))?;
        Self::merge_args(conf.try_into()?, &args)
    }

    pub fn pkg_dir(&self) -> &str {
        &self.pkg_dir
    }

    pub fn ignore_files(&self) -> &Option<HashSet<PathBuf>> {
        &self.ignore_files
    }

    pub fn ignore_paths(&self) -> &Option<HashSet<PathBuf>> {
        &self.ignore_paths
    }

    pub fn read_md5(&self) -> bool {
        self.md5
    }

    pub fn read_mtime(&self) -> bool {
        self.mtime
    }

    fn merge_args(mut s: Self, args: &ArgMatches) -> Result<Self,ConfigError> {
        s.md5   = unwrap_bool_arg!(args.value_of("md5"));
        s.mtime = unwrap_bool_arg!(args.value_of("mtime"));

        if args.occurrences_of("pkg_dir") > 0 {
            s.pkg_dir = args.value_of("pkg_dir").unwrap().to_string();
        }

        if args.is_present("ignore_paths") {
            let paths = unwrap_array_arg!(args.values_of("ignore_paths"));
            if let Some(ignore_paths) = &mut s.ignore_paths {
                ignore_paths.extend(paths);
            } else {
                s.ignore_paths = Some(paths);
            }
        }

        if args.is_present("ignore_files") {
            let files = unwrap_array_arg!(args.values_of("ignore_files"));
            if let Some(ignore_files) = &mut s.ignore_files {
                ignore_files.extend(files);
            } else {
                s.ignore_files = Some(files);
            }
        }

        Ok(s)
    }
}

fn parse_args() -> ArgMatches<'static> {
    clap::App::new(NAME)
        .version(VERSION)
        .author(AUTHOR)
        .arg(clap::Arg::with_name("pkg_dir")
             .long("pkg-dir")
             .short("d")
             .takes_value(true)
             .default_value("/var/db/pkg")
             .value_name("path")
             .help("Path to the Gentoo package database"))
        .arg(clap::Arg::with_name("md5")
             .long("md5")
             .short("m")
             .takes_value(true)
             .possible_values(&["y", "n"])
             .help("Calculate and compare MD5 sums"))
        .arg(clap::Arg::with_name("mtime")
             .long("mtime")
             .short("t")
             .takes_value(true)
             .possible_values(&["y", "n"])
             .help("Compare file modification times"))
        .arg(clap::Arg::with_name("ignore_files")
             .long("ignore-files")
             .short("f")
             .takes_value(true)
             .multiple(true)
             .value_names(&["file"])
             .help("Files to ignore when traversing the file system"))
        .arg(clap::Arg::with_name("ignore_paths")
             .long("ignore-paths")
             .short("p")
             .takes_value(true)
             .multiple(true)
             .value_names(&["path"])
             .help("Paths to ignore when traversing the file system"))
        .get_matches()
}
