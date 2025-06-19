extern crate config;

use std::path::PathBuf;
use std::env;

use clap::{Arg,ArgAction,ArgMatches};
use config::{Config,ConfigError,File};
use serde::Deserialize;

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
        let builder = Config::builder()
            .set_default("pkg_dir", "/var/db/pkg")?
            .set_default("md5", false)?
            .set_default("mtime", false)?
            .set_default("verbose", false)?
            .set_default::<&str, Option<Vec<String>>>("ignore_paths", None)?
            .set_default::<&str, Option<Vec<String>>>("ignore_files", None)?
            .add_source(File::with_name("/etc/cruft.yaml").required(false))
            .add_source(File::with_name(&home_config()).required(false));

        let conf = builder.build()?;
        Self::merge_args(conf.try_deserialize()?, &args)
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
        if args.get_flag("md5") {
            s.md5 = !s.md5;
        }

        if args.get_flag("mtime") {
            s.mtime = !s.mtime;
        }

        if args.get_flag("verbose") {
            s.verbose = true;
        }

        if let Some(pkg_dir) = args.get_one::<String>("pkg-dir") {
            s.pkg_dir = pkg_dir.clone();
        }

        if let Some(paths) = args.get_many("ignore-path") {
            s.ignore_paths = Some(paths.map(|p: &String| { PathBuf::from(p) })
                .collect());
        }

        if let Some(files) = args.get_many("ignore-file") {
            s.ignore_files = Some(files.map(|f: &String| { PathBuf::from(f) })
                .collect());
        }

        Ok(s)
    }
}

fn home_config() -> String {
    let home = env::var("HOME").unwrap();
    format!("{}/.config/cruft.yaml", home)
}

fn parse_args() -> ArgMatches {
    command!()
        .arg(Arg::new("pkg-dir").short('d').long("pkg-dir")
            .help("Path to the Gentoo package database")
            .value_parser(value_parser!(String))
            .value_name("PATH")
            .action(ArgAction::Set)
            .default_value("/var/db/pkg"))
        .arg(arg!(-m --md5   "Calculate and compare MD5 sums (inverts config setting)")
            .action(ArgAction::SetTrue))
        .arg(arg!(-t --mtime "Compare file modification times (inverts config setting)")
            .action(ArgAction::SetTrue))
        .arg(Arg::new("ignore-file").short('f').long("ignore-file")
            .help("Add file to ignore when traversing the directory tree")
            .action(ArgAction::Append)
            .value_name("FILE"))
        .arg(Arg::new("ignore-path").short('p').long("ignore-path")
            .help("Add path to ignore when traversing the directory tree")
            .action(ArgAction::Append)
            .value_name("PATH"))
        .arg(arg!(-v --verbose "Display warnings on STDERR")
            .action(ArgAction::SetTrue))
        .get_matches()
}
