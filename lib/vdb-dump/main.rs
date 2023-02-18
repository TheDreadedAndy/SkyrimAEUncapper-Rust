//!
//! @file main.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Main file for the version database dumper.
//! @bug No known bugs.
//!

use std::ffi::OsString;
use std::vec::Vec;

use versionlib::*;

struct GameLocation(usize, usize);

impl PartialEq for GameLocation {
    fn eq(
        &self,
        other: &Self
    ) -> bool {
        self.0 == other.0
    }
}

impl Eq for GameLocation {}

impl PartialOrd for GameLocation {
    fn partial_cmp(
        &self,
        other: &Self
    ) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for GameLocation {
    fn cmp(
        &self,
        other: &Self
    ) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

/// Dumps the contents of the version db to stdout.
fn main() {
    let args: Vec<OsString> = std::env::args_os().collect();
    assert!(args.len() == 2);

    let path = std::path::Path::new(&args[1]);
    let db = VersionDb::new_from_path(path);
    let mut dump = Vec::new();

    for (id, offset) in db.as_map().iter() {
        dump.push(GameLocation(*id, offset.offset()));
    }

    dump.sort();

    println!("|----ID----|--OFFSET--|");
    for GameLocation(id, offset) in dump.iter() {
        println!("| {:08} | {:08x} |", id, offset);
    }
    println!("|----------|----------|");
}
