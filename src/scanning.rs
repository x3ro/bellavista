// Bellavista
// Copyright (C) 2021  Lucas Jenß
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::fs::{self};
use std::io;
use std::os::macos::fs::MetadataExt;
use std::path::Path;

use std::cmp::Ordering;

#[derive(Debug)]
pub struct Node {
    pub size: u64,

    pub path: String,
    pub children: Option<Vec<Node>>,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        self.size.cmp(&other.size)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.size.cmp(&other.size))
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.size == other.size
    }
}

impl Eq for Node {}

fn visit_dirs(dir: &Path) -> io::Result<Node> {
    if !dir.is_dir() {
        panic!("Not a directory")
    }

    let mut size = 0;
    let mut children = vec![];

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        let metadata = fs::symlink_metadata(&path)?;
        let t = metadata.file_type();

        if t.is_dir() {
            let res = visit_dirs(&path)?;
            size += res.size;
            children.push(res);
        } else if t.is_file() {
            let mut filesize = metadata.st_size();

            if filesize == 0 {
                filesize = 1;
            }
            assert!(filesize > 0);

            size += filesize;
            children.push(Node {
                size: filesize,
                path: entry.path().to_str().unwrap().to_owned(),
                children: None,
            });
        } else {
            // Ignore symlinks
        }
    }

    children.sort();
    children.reverse();

    Ok(Node {
        size,
        path: dir.to_str().unwrap().to_owned(),
        children: if children.len() > 0 {
            Some(children)
        } else {
            None
        },
    })
}

pub fn scan(path: &Path) -> io::Result<Node> {
    let result = visit_dirs(path)?;

    println!("Total bytes: {}", result.size);
    println!("{}", &result.path);
    for node in result.children.as_ref().unwrap() {
        println!("{}    {}", node.size, node.path);
    }

    Ok(result)
}
