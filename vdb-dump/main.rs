//!
//! @file main.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Dumps the contents of the version db to stdout.
//! @bug No known bugs.
//!

fn main() {
    let args: std::vec::Vec<std::ffi::OsString> = std::env::args_os().collect();
    assert!(args.len() == 2);

    println!("|----ID----|--OFFSET--|");
    let db = sre_common::versiondb::VersionDb::new_from_path(std::path::Path::new(&args[1]));
    for sre_common::versiondb::DatabaseItem { id, addr } in db.as_vec().iter() {
        println!("| {:08} | {:08x} |", id, addr.offset());
    }
    println!("|----------|----------|");
}
