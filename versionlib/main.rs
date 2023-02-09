//!
//! @file main.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Main file for the version database dumper.
//! @bug No known bugs.
//!

#![allow(special_module_name)]

mod lib;

use std::ffi::OsString;
use lib::*;

/// Dumps the contents of the version db to stdout.
fn main() {
    let args: Vec<OsString> = std::env::args_os().collect();
    assert!(args.len() == 2);

    let path = std::path::Path::new(&args[1]);
    let db = VersionDb::new(path);

    for (id, offset) in db.by_id.iter() {
        println!("{}\t\t{:#x}", id, offset);
    }
}
