//#![deny(missing_docs)]
#![doc(html_root_url = "https://docs.rs/cstrptr/0.1.2/")]
#![cfg_attr(feature = "unstable", feature(const_raw_ptr_deref))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
mod cstr;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
mod cstring;
mod ptr;

#[cfg(not(feature = "std"))]
pub use cstr::{CStr, FromBytesWithNulError};
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub use cstring::CString;
pub use ptr::CStrPtr;
#[cfg(feature = "std")]
pub use std::ffi::{CStr, CString, FromBytesWithNulError};

#[macro_export]
macro_rules! cstr {
    ($s:expr) => {
        unsafe { $crate::CStr::from_bytes_with_nul_unchecked(concat!($s, "\0").as_bytes()) }
    };
}
