//!
//! @file log.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Implements a logging API that creates a file in the SKSE log folder based on the
//!        name of the plugin in the version structure.
//! @bug No known bugs.
//!

use core::fmt::{Arguments, Write};
use core::ffi::CStr;

use cstdio::File;
use core_util::{Later, RacyCell, StringBuffer};
use windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxA;
use windows_sys::Win32::Globalization::{WC_ERR_INVALID_CHARS, CP_UTF8, WideCharToMultiByte};
use windows_sys::Win32::System::Com::CoTaskMemFree;
use windows_sys::Win32::UI::Shell::{SHGetKnownFolderPath, FOLDERID_Documents};
use windows_sys::Win32::Foundation::{MAX_PATH, S_OK};

#[doc(hidden)]
pub use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_ICONWARNING};

use crate::SKSEPlugin_Version;

// Enumeration to determine how an error will be presented to the user.
#[doc(hidden)]
pub enum LogType {
    File,
    Window(u32),
    Both(u32)
}

/// The size of the string buffer for writing output to files. Sized to be large enough to hold the
/// max amount of text most text editors can handle on one line.
const BUF_SIZE: usize = 8192;

/// The global file we log our output to.
static LOG_FILE: Later<RacyCell<File>> = Later::new();

/// The global log buffer used to print our output.
static LOG_BUFFER: RacyCell<StringBuffer<BUF_SIZE>> = RacyCell::new(StringBuffer::new());

impl LogType {
    //
    // Attempts to write a message to the requested log types.
    //
    // The given message must be nul-terminated.
    //
    // Note that this function does not panic, since it may be called from the panic impl.
    //
    unsafe fn log(
        &self,
        msg: &[u8]
    ) -> Result<(), ()> {
        if msg[msg.len() - 1] != 0 {
            return Err(());
        }

        let win_res = match self {
            Self::Window(ico) | Self::Both(ico) => {
                let res = MessageBoxA(
                    0,
                    msg.as_ptr(),
                    SKSEPlugin_Version.name.as_ptr().cast(),
                    *ico
                );

                if res == 0 { Err(()) } else { Ok(()) }
            },
            _ => Ok(())
        };

        let log_res = match self {
            Self::File | Self::Both(_) => {
                let msg = msg.split_at(msg.len() - 1).0;
                if LOG_FILE.is_init() &&
                        (*LOG_FILE.get()).write(msg).is_ok() && (*LOG_FILE.get()).flush().is_ok() {
                    Ok(())
                } else {
                    Err(())
                }
            },
            _ => Ok(())
        };

        win_res.and(log_res)
    }
}

/// Opens a log file under the plugins name in the SKSE log directory.
pub (in crate) fn open() {
    unsafe {
        // SAFETY: Single threaded library, protected from double init by skse.
        // SAFETY: The buffer is empty, and its size is larger than MAX_PATH (260).
        (*LOG_BUFFER.get()).clear();
        (*LOG_BUFFER.get()).write_ffi(|buf| {
            assert!(buf.len() > MAX_PATH as usize);
            let mut path: windows_sys::core::PWSTR = core::ptr::null_mut();

            assert!(SHGetKnownFolderPath(&FOLDERID_Documents, 0, 0, &mut path) == S_OK);
            assert!(WideCharToMultiByte(
                CP_UTF8,
                WC_ERR_INVALID_CHARS,
                path,
                libc::wcslen(path.cast()).try_into().unwrap(), // Size in WCHARS
                buf.as_mut_ptr(),
                buf.len().try_into().unwrap(), // Size in BYTES
                core::ptr::null_mut(),
                core::ptr::null_mut()
            ) > 0);
            CoTaskMemFree(path.cast());
        });

        (&mut *LOG_BUFFER.get()).write_fmt(format_args!(
            "\\My Games\\Skyrim Special Edition\\SKSE\\{}.log",
            CStr::from_ptr(SKSEPlugin_Version.name.as_ptr()).to_str().unwrap()
        )).unwrap();

        LOG_FILE.init(RacyCell::new(File::open(
            CStr::from_bytes_until_nul((*LOG_BUFFER.get()).as_bytes_nul()).unwrap(),
            core_util::cstr!("w+b")
        ).unwrap()));

        // Clear our file path.
        (*LOG_BUFFER.get()).clear();

        // Add the BOM to the file to mark it as UTF-8.
        (*LOG_FILE.get()).write(&cstdio::UTF8_BOM).unwrap();
    }
}

// Logs a message to the requested log types.
#[doc(hidden)]
pub fn write(
    log_type: LogType,
    args: Arguments<'_>
) {
    unsafe {
        // SAFETY: This library is single threaded.
        (*LOG_BUFFER.get()).formatln(args).unwrap();
        log_type.log((*LOG_BUFFER.get()).as_bytes_nul()).unwrap();
        (*LOG_BUFFER.get()).clear();
    }
}

//
// Logs a fatal error, opening a message box as well.
//
// Called from panic, so we have to be extra careful not to panic again.
//
#[doc(hidden)]
pub fn fatal(
    log_type: LogType,
    args: Arguments<'_>
) {
    unsafe {
        // SAFETY: This library is single threaded.
        if let Err(_) = (*LOG_BUFFER.get()).formatln(args) {
            (*LOG_BUFFER.get()).clear();
            let _ = (*LOG_BUFFER.get()).write_str(
                "The plugin encountered an unknown fatal error.\n"
            );
        }

        let _ = log_type.log((*LOG_BUFFER.get()).as_bytes_nul());
        (*LOG_BUFFER.get()).clear();
    }
}

#[macro_export]
macro_rules! skse_message {
    ( $($fmt:expr),* ) => {
        $crate::log::write($crate::log::LogType::File, $crate::core::format_args!($($fmt),*));
    };
}

#[macro_export]
macro_rules! skse_warning {
    ( $($fmt:expr),* => window ) => {
        $crate::log::write(
            $crate::log::LogType::Window($crate::log::MB_ICONWARNING),
            $crate::core::format_args!($($fmt),*)
        );
    };
    ( $($fmt:expr),* => log ) => {
        $crate::log::write(
            $crate::log::LogType::File,
            $crate::core::format_args!($($fmt),*)
        );
    };
    ( $($fmt:expr),* ) => {
        $crate::log::write(
            $crate::log::LogType::Both($crate::log::MB_ICONWARNING),
            $crate::core::format_args!($($fmt),*)
        );
    };
}

#[macro_export]
macro_rules! skse_fatal {
    ( $($fmt:expr),* => window ) => {
        $crate::log::fatal(
            $crate::log::LogType::Window($crate::log::MB_ICONERROR),
            $crate::core::format_args!($($fmt),*)
        );
    };
    ( $($fmt:expr),* => log ) => {
        $crate::log::fatal(
            $crate::log::LogType::File,
            $crate::core::format_args!($($fmt),*)
        );
    };
    ( $($fmt:expr),* ) => {
        $crate::log::fatal(
            $crate::log::LogType::Both($crate::log::MB_ICONERROR),
            $crate::core::format_args!($($fmt),*)
        );
    };
}

pub use skse_message;
pub use skse_warning;
pub use skse_fatal;
