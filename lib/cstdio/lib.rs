//!
//! Implements thin wrappers around cstdio and the win32 library, allowing files
//! to be loaded as a vector of bytes or a string, and for byte strings to be written.
//!
//! With this implementation, it is possible to perform basic file IO without
//! the std crate, which means that large amounts of unnecessary code can be
//! eliminated from the final compiled binary.
//!

#![no_std]
extern crate alloc;

use core::ptr::NonNull;
use core::ffi::CStr;
use core::mem::size_of;

use alloc::vec::Vec;
use alloc::string::String;

use libc::{fopen, fread, fwrite, fseek, ferror, ftell, fflush, fclose, FILE};
use libc::{SEEK_SET, SEEK_CUR, SEEK_END};

// Oddly, libc doesn't link in msvcrt. Only the std crate does.
#[link(name = "msvcrt")]
extern "C" {}

/// A wrapper around a cstdio FILE pointer. Guaranteed to never be NULL.
#[repr(transparent)]
pub struct File(NonNull<FILE>);

/// A wrapper for the seek values used by fseek.
pub enum Seek { Set(i64), Current(i64), End(i64) }

/// The byte order mark for UTF-16
pub const UTF16LE_BOM : [u8; 2] = [0xff, 0xfe];

/// The byte order mark for UTF-8
pub const UTF8_BOM    : [u8; 3] = [0xef, 0xbb, 0xbf];

impl File {
    /// Attempts to open the file at the specified path for reading/writing.
    pub fn open(
        path: &CStr,
        mode: &CStr
    ) -> Result<File, ()> {
        unsafe {
            // SAFETY: The user has passed us valid CStr types.
            Ok(Self(NonNull::new(fopen(path.as_ptr(), mode.as_ptr())).ok_or(())?))
        }
    }

    /// Reads in data from the invoking file stream.
    ///
    /// On success, returns the number of elements read.
    pub fn read<T: Copy>(
        &mut self,
        data: &mut [T]
    ) -> Result<usize, ()> {
        unsafe {
            // SAFETY: We are using a valid C file stream. We know our data buffer is valid.
            let ret = fread(data.as_mut_ptr().cast(), size_of::<T>(), data.len(), self.0.as_ptr());
            if ferror(self.0.as_ptr()) == 0 { Ok(ret / size_of::<T>()) } else { Err(()) }
        }
    }

    /// Writes data to the current position of the invoking file stream.
    ///
    /// On error, returns the number of elements written.
    pub fn write<T: Copy>(
        &mut self,
        data: &[T]
    ) -> Result<(), usize> {
        unsafe {
            // SAFETY: We are using a valid C file stream. We know our data buffer is valid.
            let ret = fwrite(data.as_ptr().cast(), size_of::<T>(), data.len(), self.0.as_ptr());
            if ret != data.len() * size_of::<T>() { Err(ret / size_of::<T>()) } else { Ok(()) }
        }
    }

    /// Seeks to the given position in the file.
    pub fn seek(
        &mut self,
        seek: Seek
    ) -> Result<(), ()> {
        unsafe {
            // SAFETY: We have been provided a valid file and only use valid seek values.
            if 0 == match seek {
                Seek::Set(off)     => fseek(self.0.as_ptr(), off.try_into().unwrap(), SEEK_SET),
                Seek::Current(off) => fseek(self.0.as_ptr(), off.try_into().unwrap(), SEEK_CUR),
                Seek::End(off)     => fseek(self.0.as_ptr(), off.try_into().unwrap(), SEEK_END)
            } {
                Ok(())
            } else {
                Err(())
            }
        }
    }

    /// Gets the current position within the file stream.
    pub fn pos(
        &mut self
    ) -> Result<i64, ()> {
        unsafe {
            // SAFETY: We know our file stream is valid.
            let ret = ftell(self.0.as_ptr()) as i64;
            if ret < 0 { Err(()) } else { Ok(ret) }
        }
    }

    /// Flushes the internal buffer of the file stream.
    pub fn flush(
        &mut self
    ) -> Result<(), ()> {
        unsafe {
            // SAFETY: Our file stream is valid.
            if fflush(self.0.as_ptr()) == 0 { Ok(()) } else { Err(()) }
        }
    }

    /// Consumes the contents of the file, reading all of it into a vector and returning the result.
    pub fn into_vec(
        mut self
    ) -> Result<Vec<u8>, ()> {
        let mut ret = Vec::new();
        self.seek(Seek::End(0))?;
        ret.resize(self.pos()? as usize, 0);
        self.seek(Seek::Set(0))?;
        self.read(ret.as_mut_slice())?;
        return Ok(ret);
    }

    /// Consumes the contents of the file, reading all of it into a string and returning the result.
    pub fn into_string(
        self
    ) -> Result<String, ()> {
        let bytes = self.into_vec()?;
        if bytes[0..UTF16LE_BOM.len()] == UTF16LE_BOM {
            Ok(String::from_utf16_lossy(
                unsafe {
                    // SAFETY: We know the pointer and length are valid, since they belong to our
                    //         allocated vector.
                    core::slice::from_raw_parts(bytes.as_ptr().cast(), bytes.len() / 2)
                }
            ))
        } else if bytes[0..UTF8_BOM.len()] == UTF8_BOM {
            Ok(String::from(String::from_utf8_lossy(&bytes)))
        } else {
            // Assume its utf8. If that fails, try utf16. Give up if neither works.
            match String::from_utf8(bytes) {
                Ok(s)    => Ok(s),
                Err(err) => {
                    let bytes = err.as_bytes();
                    String::from_utf16(
                        unsafe {
                            // SAFETY: We know the pointer and length are valid, since they 
                            //         belong to our allocated vector.
                            core::slice::from_raw_parts(bytes.as_ptr().cast(), bytes.len() / 2)
                        }
                    ).map_err(|_e| ())
                }
            }
        }
    }
}

impl core::fmt::Write for File {
    fn write_str(
        &mut self,
        s: &str
    ) -> core::fmt::Result {
        self.write(s.as_bytes()).map_err(|_| core::fmt::Error)?;
        self.flush().map_err(|_| core::fmt::Error)
    }
}

impl Drop for File {
    fn drop(
        &mut self
    ) {
        unsafe {
            // SAFETY: The new() function ensures our file stream is always a valid pointer.
            assert!(fclose(self.0.as_ptr()) == 0);
        }
    }
}
