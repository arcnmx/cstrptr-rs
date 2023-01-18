#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::borrow::Cow;
#[cfg(feature = "std")]
use std::borrow::Cow;
use {
    crate::{CStr, FromBytesWithNulError},
    core::marker::PhantomData,
    cty::c_char,
};

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct CStrPtr<'a> {
    inner: *const c_char,
    marker: PhantomData<&'a [c_char]>,
}

impl<'a> CStrPtr<'a> {
    #[inline]
    pub const fn from_c_str(str: &'a CStr) -> Self {
        unsafe { Self::from_ptr(str.as_ptr()) }
    }

    #[inline]
    pub const unsafe fn from_ptr(ptr: *const c_char) -> Self {
        CStrPtr {
            inner: ptr,
            marker: PhantomData,
        }
    }

    #[inline]
    pub fn from_bytes_with_nul(bytes: &'a [u8]) -> Result<Self, FromBytesWithNulError> {
        CStr::from_bytes_with_nul(bytes).map(From::from)
    }

    #[inline]
    pub const unsafe fn from_bytes_with_nul_unchecked(bytes: &'a [u8]) -> Self {
        Self::from_ptr(bytes.as_ptr() as *const _)
    }

    #[inline]
    pub const fn as_ptr(self) -> *const c_char {
        self.inner
    }

    #[inline]
    pub fn to_bytes(self) -> &'a [u8] {
        self.to_c_str().to_bytes()
    }

    pub fn to_bytes_with_nul(self) -> &'a [u8] {
        self.to_c_str().to_bytes_with_nul()
    }

    #[inline]
    pub fn to_c_str(self) -> &'a CStr {
        unsafe { CStr::from_ptr(self.inner) }
    }

    #[inline]
    pub fn to_str(self) -> Result<&'a str, core::str::Utf8Error> {
        // TODO: inefficient
        core::str::from_utf8(self.to_bytes())
    }

    #[inline]
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub fn to_string_lossy(self) -> Cow<'a, str> {
        self.to_c_str().to_string_lossy()
    }
}

unsafe impl<'a> Send for CStrPtr<'a> {}
unsafe impl<'a> Sync for CStrPtr<'a> {}

impl<'a> PartialEq for CStrPtr<'a> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.to_bytes().eq(other.to_bytes())
    }
}

impl<'a> Eq for CStrPtr<'a> {}

impl<'a> PartialOrd for CStrPtr<'a> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.to_bytes().partial_cmp(&other.to_bytes())
    }
}

impl<'a> Ord for CStrPtr<'a> {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.to_bytes().cmp(&other.to_bytes())
    }
}

impl<'a> AsRef<CStr> for CStrPtr<'a> {
    // NOTE: this isn't really a cheap conversion...
    #[inline]
    fn as_ref(&self) -> &CStr {
        self.to_c_str()
    }
}

impl<'a> From<CStrPtr<'a>> for &'a CStr {
    #[inline]
    fn from(ptr: CStrPtr<'a>) -> Self {
        ptr.to_c_str()
    }
}

impl<'a> From<&'a CStr> for CStrPtr<'a> {
    #[inline]
    fn from(ptr: &'a CStr) -> Self {
        Self::from_c_str(ptr)
    }
}
