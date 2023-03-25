//!
//! @file log.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Implements a logging API that creates a file in the SKSE log folder based on the
//!        name of the plugin in the version structure.
//! @bug No known bugs.
//!

use std::fmt;
use std::fmt::Arguments;
use std::fs::File;
use std::io::Write;
use std::ffi::{CStr, OsString};
use std::os::windows::ffi::{OsStrExt, OsStringExt};

use later::Later;
use racy_cell::RacyCell;
use windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxW;
use windows_sys::Win32::UI::Shell::{SHGetFolderPathW, CSIDL_MYDOCUMENTS, SHGFP_TYPE_CURRENT};
use windows_sys::Win32::Foundation::MAX_PATH;

#[doc(hidden)]
pub use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_ICONWARNING};

use crate::loader::SKSEPlugin_Version;

///
/// The structure used to format information before writing it to the log file.
/// We use this to avoid an allocation for each message.
///
/// The buffer is always ended with a null terminator. Note that the buffer is
/// encoded in UTF-16, as this is the format that windows actually uses for its
/// OS strings.
///
struct LogBuf {
    buf: [u16; Self::BUF_SIZE],
    len: usize
}

// Enumeration to determine how an error will be presented to the user.
#[doc(hidden)]
pub enum LogType {
    File,
    Window(u32),
    Both(u32)
}

/// The global file we log our output to.
static LOG_FILE: Later<RacyCell<File>> = Later::new();

/// The global log buffer used to print our output.
static LOG_BUFFER: RacyCell<LogBuf> = RacyCell::new(LogBuf::new());

/// The OS-encoded name of our plugin.
static OS_PLUGIN_NAME: Later<Vec<u16>> = Later::new();

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

    /// Gets the underlying &[u16] in the buffer, excluding the null.
    fn as_bytes(
        &self
    ) -> &[u16] {
        self.buf.split_at(self.len).0
    }

    /// Gets the underlying &[u16] in the buffer, with the null.
    fn as_bytes_nul(
        &self
    ) -> &[u16] {
        self.buf.split_at(self.len + 1).0
    }

    /// Formats the given arguments into the buffer, adding a newline.
    fn formatln(
        &mut self,
        args: Arguments<'_>
    ) -> Result<(), std::fmt::Error> {
        fmt::write(self, args)?;
        <dyn fmt::Write>::write_str(self, "\n")?;
        Ok(())
    }

    ///
    /// Calls the given function, then updates the length of the buffer based on the null
    /// terminator.
    ///
    /// The given function must null terminate any data it appends.
    ///
    unsafe fn write_ffi(
        &mut self,
        func: impl FnOnce(&mut [u16])
    ) {
        func(self.buf.split_at_mut(self.len).1);

        for c in self.buf.split_at(self.len).1.iter() {
            if *c == 0 {
                return;
            }

            self.len += 1;
        }
    }

    /// Erases the contents of the buffer.
    fn clear(
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
        for c in s.encode_utf16() {
            self.buf[self.len] = c;
            if self.len < Self::BUF_SIZE - 1 {
                self.len += 1;
            }
        }
        self.buf[self.len] = 0; // Always null terminate.
        Ok(())
    }
}

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
        msg: &[u16]
    ) -> Result<(), ()> {
        if msg[msg.len() - 1] != 0 {
            return Err(());
        }

        let win_res = match self {
            Self::Window(ico) | Self::Both(ico) => {
                let res = MessageBoxW(
                    0,
                    msg.as_ptr(),
                    OS_PLUGIN_NAME.as_ptr().cast(),
                    *ico
                );

                if res == 0 { Err(()) } else { Ok(()) }
            },
            _ => Ok(())
        };

        let log_res = match self {
            Self::File | Self::Both(_) => {
                let msg = msg.split_at(msg.len() - 1).0;
                let msg: &[u8] = std::slice::from_raw_parts(
                    msg.as_ptr().cast(),
                    msg.len() * std::mem::size_of::<u16>()
                );
                if LOG_FILE.is_init() &&
                        (*LOG_FILE.get()).write(msg).is_ok() {
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

/// Opens a log file with the given name in the SKSE log directory.
pub (in crate) fn open() {
    unsafe {
        // SAFETY: Single threaded library, protected from double init by skse.
        // SAFETY: The buffer is empty, and its size is larger than MAX_PATH (260).
        (*LOG_BUFFER.get()).clear();
        (*LOG_BUFFER.get()).write_ffi(|buf| {
            assert!(buf.len() > MAX_PATH as usize);
            SHGetFolderPathW(
                0,
                CSIDL_MYDOCUMENTS as i32,
                0,
                SHGFP_TYPE_CURRENT as u32,
                buf.as_mut_ptr()
            );
        });

        let plugin_name = CStr::from_ptr(SKSEPlugin_Version.name.as_ptr()).to_str().unwrap();
        OS_PLUGIN_NAME.init(OsString::from(plugin_name.to_string()).encode_wide().collect());

        <dyn fmt::Write>::write_fmt(&mut *LOG_BUFFER.get(), format_args!(
            "\\My Games\\Skyrim Special Edition\\SKSE\\{}.log",
            plugin_name
        )).unwrap();

        LOG_FILE.init(RacyCell::new(
            File::create(&OsString::from_wide((*LOG_BUFFER.get()).as_bytes())).unwrap()
        ));

        // Clear our file path.
        (*LOG_BUFFER.get()).clear();

        // Write the byte-order mark, so text editors know the file is UTF-16.
        (*LOG_FILE.get()).write(&[0xFF, 0xFE]).unwrap();
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
            <dyn fmt::Write>::write_str(
                &mut *LOG_BUFFER.get(),
                "The plugin encountered an unknown fatal error.\n"
            ).unwrap_unchecked();
        }

        let _ = log_type.log((*LOG_BUFFER.get()).as_bytes_nul());
        (*LOG_BUFFER.get()).clear();
    }
}

#[macro_export]
macro_rules! skse_message {
    ( $($fmt:expr),* ) => {
        $crate::log::write($crate::log::LogType::File, ::std::format_args!($($fmt),*));
    };
}

#[macro_export]
macro_rules! skse_warning {
    ( $($fmt:expr),* => window ) => {
        $crate::log::write(
            $crate::log::LogType::Window($crate::log::MB_ICONWARNING),
            ::std::format_args!($($fmt),*)
        );
    };
    ( $($fmt:expr),* => log ) => {
        $crate::log::write(
            $crate::log::LogType::File,
            ::std::format_args!($($fmt),*)
        );
    };
    ( $($fmt:expr),* ) => {
        $crate::log::write(
            $crate::log::LogType::Both($crate::log::MB_ICONWARNING),
            ::std::format_args!($($fmt),*)
        );
    };
}

#[macro_export]
macro_rules! skse_fatal {
    ( $($fmt:expr),* => window ) => {
        $crate::log::fatal(
            $crate::log::LogType::Window($crate::log::MB_ICONERROR),
            ::std::format_args!($($fmt),*)
        );
    };
    ( $($fmt:expr),* => log ) => {
        $crate::log::fatal(
            $crate::log::LogType::File,
            ::std::format_args!($($fmt),*)
        );
    };
    ( $($fmt:expr),* ) => {
        $crate::log::fatal(
            $crate::log::LogType::Both($crate::log::MB_ICONERROR),
            ::std::format_args!($($fmt),*)
        );
    };
}

pub use skse_message;
pub use skse_warning;
pub use skse_fatal;
