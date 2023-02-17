//!
//! @file query.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Allows the plugin to query information about the running skyrim process.
//! @bug No known bugs.
//!

use std::ptr;
use std::mem::size_of;
use std::collections::HashSet;
use std::vec::Vec;
use std::str::FromStr;
use std::str;

use windows_sys::Win32::System::ProcessStatus::{K32EnumProcessModulesEx, K32GetModuleBaseNameA};
use windows_sys::Win32::System::ProcessStatus::LIST_MODULES_ALL;
use windows_sys::Win32::System::Threading::GetCurrentProcess;
use windows_sys::Win32::Foundation::{HANDLE, MAX_PATH};

pub fn loaded_plugins() -> HashSet<String> {
    let mut ret = HashSet::new();

    unsafe {
        // SAFETY: We ensure that we only give these function properly allocated and sized
        //         pointers.

        // Get the number of modules loaded by Skyrim.
        let proc = GetCurrentProcess();
        let mut mod_bytes: u32 = 0;
        assert!(K32EnumProcessModulesEx(
            proc,
            ptr::null_mut(),
            0,
            &mut mod_bytes,
            LIST_MODULES_ALL
        ) != 0);

        // Get the list of modules.
        let mut modules: Vec<HANDLE> = Vec::new();
        modules.resize(mod_bytes as usize / size_of::<HANDLE>(), 0);
        assert!(K32EnumProcessModulesEx(
            proc,
            modules.as_mut_ptr(),
            mod_bytes,
            &mut mod_bytes,
            LIST_MODULES_ALL
        ) != 0);

        // Add each module to the set.
        for module in modules.iter() {
            let mut name: [u8; MAX_PATH as usize + 1] = [0; MAX_PATH as usize + 1];
            let len = K32GetModuleBaseNameA(proc, *module, name.as_mut_ptr(), MAX_PATH + 1);
            assert!((0 < len) && (len <= MAX_PATH));

            let len = len as usize;
            ret.insert(String::from_str(str::from_utf8(name.split_at(len).0).unwrap()).unwrap());
        }
    }

    return ret;
}
