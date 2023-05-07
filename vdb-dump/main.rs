//!
//! @file main.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Main file for the version database dumper.
//! @bug No known bugs.
//!

use std::ffi::OsString;
use std::vec::Vec;

use versionlib::*;

/// Dumps the contents of the version db to stdout.
fn main() {
    let args: Vec<OsString> = std::env::args_os().collect();
    assert!(args.len() == 2);

    let path = std::path::Path::new(&args[1]);
    let db = VersionDb::new_from_path(path);

    println!("|----ID----|--OFFSET--|");
    for DatabaseItem { id, addr } in db.as_vec().iter() {
        println!("| {:08} | {:08x} |", id, addr.offset());
    }
    println!("|----------|----------|");
}
