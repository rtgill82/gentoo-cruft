# gentoo-cruft

_Find files that are not recorded in the Gentoo package database._

Scans the file system and compares it to the `CONTENTS` catalogs in the Gentoo
installed package database. Files that are not listed in the package database
are displayed. MD5 sums and mtimes can also be calculated in order to list
files that have been modified since installation.

## Usage

Just running `cruft` at the command line will perform a simple comparison of
installed files. Specifying `--md5` or `--mtime` will check for modified files.
Full command line parameters are as follows:

```
USAGE:
    cruft [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -m, --md5        Calculate and compare MD5 sums (inverts config setting)
    -t, --mtime      Compare file modification times (inverts config setting)
    -V, --version    Prints version information
    -v, --verbose    Display warnings on STDERR

OPTIONS:
    -f, --ignore-files <file>...    Files to ignore when traversing the directory tree
    -p, --ignore-paths <path>...    Paths to ignore when traversing the directory tree
    -d, --pkg-dir <path>            Path to the Gentoo package database [default: /var/db/pkg]
```

## Configuration

The configuration files `/etc/cruft.yaml` and `$HOME/.config/cruft.yaml` will
be read if they are available. An example is provided in the crate or in the
repository at:
https://github.com/rtgill82/gentoo-cruft/blob/master/config/cruft.yaml

## LICENSE

Copyright (C) 2020,2025 Robert Gill <<rtgill82@gmail.com>>

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to
deal in the Software without restriction, including without limitation the
rights to use, copy, modify, merge, publish, distribute, sublicense, and/or
sell copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies of the Software, its documentation and marketing & publicity
materials, and acknowledgment shall be given in the documentation, materials
and software packages that this Software was used.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
