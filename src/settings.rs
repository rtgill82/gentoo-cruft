//
// Copyright (C) 2020,2025 Robert Gill <rtgill82@gmail.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to
// deal in the Software without restriction, including without limitation the
// rights to use, copy, modify, merge, publish, distribute, sublicense, and/or
// sell copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies of the Software, its documentation and marketing & publicity
// materials, and acknowledgment shall be given in the documentation, materials
// and software packages that this Software was used.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
// THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
// IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
// CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
//

#![allow(static_mut_refs)]

use std::mem::MaybeUninit;
use std::path::PathBuf;
use std::sync::{Arc,Once};
use std::{env,fs,process};

use clap::{Arg,ArgAction,ArgMatches};
use config::{Config,ConfigError,File};
use serde::Deserialize;

use crate::symlink::Symlink;

static START: Once = Once::new();
static mut INSTANCE: MaybeUninit<Arc<Settings>> = MaybeUninit::uninit();

#[derive(Debug,Default,Deserialize)]
pub struct Settings {
    pkg_dir: String,
    ignore_files: Option<Vec<PathBuf>>,
    ignore_paths: Option<Vec<PathBuf>>,
    links_to_usr: Option<Vec<Symlink>>,
    split_usr: bool,
    md5: bool,
    mtime: bool,
    verbose: bool
}

impl Settings {
    pub fn get() -> Arc<Settings> {
        START.call_once(|| {
            match Self::init() {
                Ok(settings) => {
                    let settings = Arc::new(settings);
                    unsafe { INSTANCE.write(settings); }
                },

                Err(err) => {
                    eprintln!("Error reading configuration: {}", err);
                    process::exit(1);
                }
            }
        });

        unsafe { (*INSTANCE.as_ptr()).clone() }
    }

    fn init() -> Result<Self,ConfigError> {
        let args = parse_args();
        let builder = Config::builder()
            .set_default("pkg_dir", "/var/db/pkg")?
            .set_default("split_usr", false)?
            .set_default("md5", false)?
            .set_default("mtime", false)?
            .set_default("verbose", false)?
            .set_default::<&str, Option<Vec<String>>>("ignore_paths", None)?
            .set_default::<&str, Option<Vec<String>>>("ignore_files", None)?
            .add_source(File::with_name("/etc/cruft.yaml").required(false))
            .add_source(File::with_name(&home_config()).required(false));

        let conf = builder.build()?;
        let mut settings = Self::merge_args(conf.try_deserialize()?, &args);
        settings.links_to_usr = read_links();
        settings.split_usr = is_split_usr(settings.links_to_usr.is_some());
        Ok(settings)
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

    pub fn links_to_usr(&self) -> Option<&Vec<Symlink>> {
        self.links_to_usr.as_ref()
    }

    pub fn is_split_usr(&self) -> bool {
        self.split_usr
    }

    pub fn md5(&self) -> bool {
        self.md5
    }

    pub fn mtime(&self) -> bool {
        self.mtime
    }

    pub fn verbose(&self) -> bool {
        self.verbose
    }

    fn merge_args(mut settings: Self, args: &ArgMatches) -> Self {
        if args.get_flag("md5") {
            settings.md5 = !settings.md5;
        }

        if args.get_flag("mtime") {
            settings.mtime = !settings.mtime;
        }

        if args.get_flag("verbose") {
            settings.verbose = true;
        }

        if let Some(pkg_dir) = args.get_one::<String>("pkg-dir") {
            settings.pkg_dir = pkg_dir.clone();
        }

        if let Some(paths) = args.get_many("ignore-path") {
            settings.ignore_paths = Some(paths.map(|p: &String| {
                PathBuf::from(p)
            })
                .collect());
        }

        if let Some(files) = args.get_many("ignore-file") {
            settings.ignore_files = Some(files.map(|f: &String| {
                PathBuf::from(f)
            })
                .collect());
        }

        settings
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

fn is_split_usr(links_exist: bool) -> bool {
    does_profile_contain_split_usr() && !links_exist
}

fn does_profile_contain_split_usr() -> bool {
    let link = fs::read_link("/etc/portage/make.profile")
        .expect("Unable to read `/etc/portage/make.profile`");

    let os_str = link.as_os_str();
    let link_str = os_str.to_str()
        .expect("Path contains invalid Unicode");
    if link_str.contains("/split-usr") {
        return true;
    }

    false
}

fn read_links() -> Option<Vec<Symlink>> {
    let mut links: Option<Vec<Symlink>> = None;
    for path in ["/bin", "/lib", "/lib64", "/sbin"] {
        let link = match fs::read_link(path) {
            Ok(link) => PathBuf::from("/").join(link),
            Err(_) => continue
        };

        let path = PathBuf::from(path);
        match &mut links {
            Some(vec) => vec.push(Symlink::new(path, link)),
            None => links = Some(Vec::from(&[Symlink::new(path, link)]))
        }
    }

    if let Ok(link) = fs::read_link("/usr/sbin") {
        let link = if link.components().count() == 1 {
            Some(PathBuf::from("/usr").join(link))
        } else if link.starts_with("/usr") {
            Some(PathBuf::from(link))
        } else {
            None
        };

        if let Some(link) = link {
            let path = PathBuf::from("/usr/sbin");

            match &mut links {
                Some(vec) => vec.push(Symlink::new(path, link)),
                None => links = Some(Vec::from(&[Symlink::new(path, link)]))
            }
        }
    }

    links
}
