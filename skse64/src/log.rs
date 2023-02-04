//!
//! @file log.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Wraps the SKSE logging API.
//! @bug No known bugs.
//!

use std::fmt;
use std::fmt::Arguments;
use std::fs::File;
use std::io::Write;

use later::Later;
use racy_cell::RacyCell;
use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxA, MB_ICONERROR};
use windows_sys::Win32::UI::Shell::{SHGetFolderPathA, CSIDL_MYDOCUMENTS, SHGFP_TYPE_CURRENT};

use crate::SKSEPlugin_Version;

///
/// The structure used to format information before writing it to the log file.
///
/// We use this to avoid an allocation for each message.
///
/// The buffer is always ended with a null terminator.
///
struct LogBuf {
    buf: [u8; Self::BUF_SIZE],
    len: usize
}

/// The global file we log our output to.
static LOG_FILE: Later<RacyCell<File>> = Later::new();

/// The global log buffer used to print our output.
static LOG_BUFFER: RacyCell<LogBuf> = RacyCell::new(LogBuf::new());

impl LogBuf {
    /// Large enough to contain any reasonably size line in a log file.
    const BUF_SIZE: usize = 8192;

    /// Creates a new, empty, log buffer.
    const fn new() -> Self {
        Self {
            buf: [0; Self::BUF_SIZE],
            len: 0
        }
    }

    /// Gets the current length of the buffer.
    const fn len(
        &self
    ) -> usize {
        self.len
    }

    /// Gets the underlying &[u8] in the buffer, excluding the null.
    fn as_bytes(
        &self
    ) -> &[u8] {
        self.buf.split_at(self.len).0
    }

    /// Gets a mutable pointer to the buffer.
    unsafe fn as_mut_ptr(
        &mut self
    ) -> *mut u8 {
        self.buf.as_mut_ptr()
    }

    ///
    /// Updates the buffer length to point to the null terminator.
    ///
    /// For use after FFI has written into the buffer.
    ///
    fn update_len(
        &mut self
    ) {
        for c in self.buf.split_at(self.len).1.iter() {
            if *c == 0 {
                return;
            }

            self.len += 1;
        }
    }

    /// Erases the contents of the buffer.
    fn flush(
        &mut self
    ) {
        self.buf[0] = 0;
        self.len = 0;
    }
}

impl fmt::Write for LogBuf {
    fn write_str(
        &mut self,
        s: &str
    ) -> Result<(), fmt::Error> {
        for c in s.as_bytes().iter() {
            self.buf[self.len] = *c;
            if self.len < Self::BUF_SIZE - 1 {
                self.len += 1;
            }
        }
        self.buf[self.len] = 0; // Always null terminate.
        Ok(())
    }
}

/// Opens a log file with the given name in the SKSE log directory.
pub (in crate) fn open() {
    unsafe {
        // SAFETY: Single threaded library, protected from double init by skse.
        // SAFETY: The buffer is empty, and its size is larger than MAX_PATH (260).
        (*LOG_BUFFER.get()).flush();
        SHGetFolderPathA(
            0,
            CSIDL_MYDOCUMENTS as i32,
            0,
            SHGFP_TYPE_CURRENT as u32,
            (*LOG_BUFFER.get()).as_mut_ptr()
        );
        (*LOG_BUFFER.get()).update_len();

        <dyn fmt::Write>::write_fmt(&mut *LOG_BUFFER.get(), format_args!(
            "\\My Games\\Skyrim Special Edition\\SKSE\\{}.log",
            std::ffi::CStr::from_ptr(SKSEPlugin_Version.name.as_ptr()).to_str().unwrap()
        )).unwrap();

        LOG_FILE.init(RacyCell::new(
            File::create(std::str::from_utf8((*LOG_BUFFER.get()).as_bytes()).unwrap()).unwrap()
        ));
    }
}

// Writes out the given format string to the opened log file.
#[doc(hidden)]
pub fn write(
    args: Arguments<'_>
) {
    unsafe {
        // SAFETY: This library is single threaded.
        fmt::write(LOG_BUFFER.get().as_mut().unwrap_unchecked(), args).unwrap();
        <dyn fmt::Write>::write_str(&mut *LOG_BUFFER.get(), "\n").unwrap();
        assert!((*LOG_FILE.get()).write(
            (*LOG_BUFFER.get()).as_bytes()
        ).unwrap() == (*LOG_BUFFER.get()).len());
        (*LOG_BUFFER.get()).flush();
    }
}

//
// Logs a fatal error, opening a message box as well.
//
// Called from panic, so we have to be extra careful not to panic again.
//
#[doc(hidden)]
pub fn fatal(
    args: Arguments<'_>
) {
    unsafe {
        // SAFETY: This library is single threaded.
        let msg = if let Ok(_) = fmt::write(LOG_BUFFER.get().as_mut().unwrap_unchecked(), args) {
            // Try to add a newline.
            let _ = <dyn fmt::Write>::write_str(&mut *LOG_BUFFER.get(), "\n");

            (*LOG_BUFFER.get()).as_bytes()
        } else {
            "The plugin encountered an unknown fatal error.\n\0".as_bytes()
        };

        // Attempt to show a message box and print it to the log.
        MessageBoxA(0, msg.as_ptr(), SKSEPlugin_Version.name.as_ptr().cast(), MB_ICONERROR);
        if LOG_FILE.is_init() {
            let _ = (*LOG_FILE.get()).write(msg.split_at(msg.len() - 1).0);
        }
    }
}

#[macro_export]
macro_rules! skse_message {
    ( $($fmt:tt)* ) => {
        $crate::log::write(::std::format_args!($($fmt)*));
    };
}

#[macro_export]
macro_rules! skse_error {
    ( $($fmt:tt)* ) => {
        $crate::log::write(::std::format_args!($($fmt)*));
    };
}

#[macro_export]
macro_rules! skse_fatal {
    ( $($fmt:tt)* ) => {
        $crate::log::fatal(::std::format_args!($($fmt)*));
    };
}

pub use skse_message;
pub use skse_error;
pub use skse_fatal;
