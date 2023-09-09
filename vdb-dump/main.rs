//!
//! @file main.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Dumps the contents of the version db to stdout.
//! @bug No known bugs.
//!

use sre_common::versiondb::{VersionDbStream, DatabaseItem};

fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    assert!(args.len() == 2);

    // Terminate the arg string.
    args[1] += "\0";

    // We can't use the VersionDb until after the relocation manager has some base address. The SKSE
    // loading code in libskyrim normally handles this, but we're not a skyrim mod so we need to do
    // it ourself.
    sre_common::skse64::reloc::RelocAddr::init_manager(0x140000000);

    println!("|----ID----|--OFFSET--|");
    let mut db: Vec<DatabaseItem> = VersionDbStream::new_from_path(
        std::ffi::CStr::from_bytes_until_nul(args[1].as_bytes()).unwrap()
    ).collect();
    db.sort_by(|lhs, rhs| lhs.id.cmp(&rhs.id));

    for DatabaseItem { id, addr } in db.iter() {
        println!("| {:08} | {:08x} |", id, addr.offset());
    }
    println!("|----------|----------|");
}
