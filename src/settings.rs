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

#[derive(Debug,Default,Deserialize)]
pub struct Settings {
    pkg_dir: String,
    ignore_files: Option<Vec<PathBuf>>,
    ignore_paths: Option<Vec<PathBuf>>,
    md5: bool,
    mtime: bool,
    verbose: bool
}

impl Settings {
    pub fn new() -> Result<Self,ConfigError> {
        let args = parse_args();
        let mut conf = Config::new();

        conf.set_default("md5", false)?;
        conf.set_default("mtime", false)?;
        conf.set_default("verbose", false)?;
        conf.set_default::<Option<Vec<String>>>("ignore_paths", None)?;
        conf.set_default::<Option<Vec<String>>>("ignore_files", None)?;
        conf.merge(File::with_name("config/cruft.yaml"))?;
        Self::merge_args(conf.try_into()?, &args)
    }

    pub fn pkg_dir(&self) -> &str {
        &self.pkg_dir
    }

    pub fn ignore_files(&self) -> Option<&Vec<PathBuf>> {
        self.ignore_files.as_ref()
    }

    pub fn ignore_paths(&self) -> Option<&Vec<PathBuf>> {
        self.ignore_paths.as_ref()
    }

    pub fn read_md5(&self) -> bool {
        self.md5
    }

    pub fn read_mtime(&self) -> bool {
        self.mtime
    }

    pub fn verbose(&self) -> bool {
        self.verbose
    }

    fn merge_args(mut s: Self, args: &ArgMatches) -> Result<Self,ConfigError> {
        if args.occurrences_of("md5") > 0 {
            s.md5 = !s.md5;
        }

        if args.occurrences_of("mtime") > 0 {
            s.mtime = !s.mtime;
        }

        if args.occurrences_of("verbose") > 0 {
            s.verbose = true;
        }

        if args.occurrences_of("pkg_dir") > 0 {
            s.pkg_dir = args.value_of("pkg_dir").unwrap().to_string();
        }

        if args.is_present("ignore_paths") {
            let mut paths = unwrap_array_arg!(args.values_of("ignore_paths"));
            if let Some(ignore_paths) = s.ignore_paths {
                paths.extend(ignore_paths);
            }
            s.ignore_paths = Some(paths.into_iter().collect());
        }

        if args.is_present("ignore_files") {
            let mut files = unwrap_array_arg!(args.values_of("ignore_files"));
            if let Some(ignore_files) = s.ignore_files {
                files.extend(ignore_files);
            }
            s.ignore_files = Some(files.into_iter().collect());
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
             .help("Calculate and compare MD5 sums (inverses config setting)"))
        .arg(clap::Arg::with_name("mtime")
             .long("mtime")
             .short("t")
             .help("Compare file modification times (inverses config setting)"))
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
        .arg(clap::Arg::with_name("verbose")
             .long("verbose")
             .short("v")
             .help("Display warnings on STDERR"))
        .get_matches()
}
