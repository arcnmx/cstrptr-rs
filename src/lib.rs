//#![deny(missing_docs)]
#![doc(html_root_url = "http://docs.rs/cstrptr/0.1.1")]
#![cfg_attr(feature = "unstable", feature(const_raw_ptr_deref, const_fn))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
mod cstring;
#[cfg(not(feature = "std"))]
mod cstr;
mod ptr;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub use cstring::CString;
#[cfg(not(feature = "std"))]
pub use cstr::{CStr, FromBytesWithNulError};
pub use ptr::CStrPtr;

#[cfg(feature = "std")]
pub use std::ffi::{CStr, FromBytesWithNulError, CString};

#[macro_export]
macro_rules! cstr {
    ($s:expr) => {
        unsafe {
            $crate::CStr::from_bytes_with_nul_unchecked(concat!($s, "\0").as_bytes())
        }
    };
}
