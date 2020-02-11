# gentoo-cruft

_Find files not recorded in the Gentoo package database._

Scans the file system and compares it to the `CONTENTS` catalogs in the Gentoo
installed package database. Files that are not listed in the package database
are listed. MD5 sums and mtimes can also be calculated in order to list files
that have been modified since installation.

## Usage

Just running `cruft` at the command line will perform a simple comparison of
installed files. Specifying `--md5` or `--mtime` will check for modified files.
Full command line parameters are as follows:

```
USAGE:
    cruft [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -m, --md5        Calculate and compare MD5 sums (inverses config setting)
    -t, --mtime      Compare file modification times (inverses config setting)
    -V, --version    Prints version information
    -v, --verbose    Display warnings on STDERR

OPTIONS:
    -f, --ignore-files <file>...    Files to ignore when traversing the file system
    -p, --ignore-paths <path>...    Paths to ignore when traversing the file system
    -d, --pkg-dir <path>            Path to the Gentoo package database [default: /var/db/pkg]
```

## Configuration

The configuration files `/etc/cruft.yaml` and `$HOME/.config/cruft.yaml` will
be read if they are available. An example is provided in the crate or in the
repository at:
https://github.com/xelkarin/gentoo-cruft/blob/master/config/cruft.yaml
