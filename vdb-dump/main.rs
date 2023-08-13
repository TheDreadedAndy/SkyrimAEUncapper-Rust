//!
//! @file main.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Dumps the contents of the version db to stdout.
//! @bug No known bugs.
//!

fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    assert!(args.len() == 2);

    // Terminate the arg string.
    args[1] += "\0";

    println!("|----ID----|--OFFSET--|");
    let db = sre_common::versiondb::VersionDb::new_from_path(
        std::ffi::CStr::from_bytes_until_nul(args[1].as_bytes()).unwrap()
    );
    for sre_common::versiondb::DatabaseItem { id, addr } in db.as_vec().iter() {
        println!("| {:08} | {:08x} |", id, addr.offset());
    }
    println!("|----------|----------|");
}
