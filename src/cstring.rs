use core::ops;
use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::string::String;
use cty::c_char;
use crate::{ CStr, CStrPtr };

#[derive(Clone)]
pub struct CString {
    inner: Box<[u8]>,
}

impl CString {
    pub fn new<T: Into<Vec<u8>>>(bytes: T) -> Result<Self, Vec<u8>> {
        let bytes = bytes.into();
        match CStr::from_bytes_with_nul(&bytes) {
            Err(crate::FromBytesWithNulError::NotNulTerminated) =>
                Ok(unsafe { Self::from_bytes_with_nul_unchecked(bytes) }),
            _ => Err(bytes), // TODO: NulError
        }
    }

    pub fn from_bytes_with_nul<T: Into<Box<[u8]>>>(bytes: T) -> Result<Self, Box<[u8]>> {
        let bytes = bytes.into();
        match CStr::from_bytes_with_nul(&bytes) {
            Ok(_) => Ok(unsafe { Self::from_bytes_with_nul_unchecked(bytes) }),
            Err(e) => Err(bytes), // TODO: NulError
        }
    }

    pub unsafe fn from_vec_unchecked(mut v: Vec<u8>) -> Self {
        v.reserve_exact(1);
        v.push(0);
        Self::from_bytes_with_nul_unchecked(v)
    }

    #[inline]
    pub unsafe fn from_bytes_with_nul_unchecked<T: Into<Box<[u8]>>>(bytes: T) -> Self {
        Self { inner: bytes.into() }
    }

    pub unsafe fn from_raw(ptr: *mut c_char) -> Self {
        let len = CStr::from_ptr(ptr).to_bytes_with_nul().len();
        let slice = core::slice::from_raw_parts_mut(ptr, len);
        Self { inner: Box::from_raw(slice as *mut [_] as *mut [u8]) }
    }

    #[inline]
    pub fn into_raw(self) -> *mut c_char {
        Box::into_raw(self.inner) as *mut c_char
    }

    pub fn into_string(self) -> Result<String, Self> {
        String::from_utf8(self.into_bytes()).map_err(|e|
            // TODO: custom error type
            unsafe { Self::from_vec_unchecked(e.into_bytes()) }
        )
    }

    pub fn into_bytes(self) -> Vec<u8> {
        let mut vec = self.inner.into_vec();
        let _nul = vec.pop();
        debug_assert_eq!(_nul, Some(0u8));
        vec
    }

    #[inline]
    pub fn into_bytes_with_nul(self) -> Vec<u8> {
        self.inner.into_vec()
    }

    #[inline]
    pub fn as_c_str(&self) -> &CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(&self.inner[..]) }
    }

    #[inline]
    pub fn into_boxed_c_str(self) -> Box<CStr> {
        unsafe { Box::from_raw(Box::into_raw(self.inner) as *mut CStr) }
    }
}

impl ops::Index<ops::RangeFull> for CString {
    type Output = CStr;

    #[inline]
    fn index(&self, _index: ops::RangeFull) -> &CStr {
        self.as_c_str()
    }
}

impl From<&CStr> for CString {
    #[inline]
    fn from(s: &CStr) -> Self {
        Self { inner: s.to_bytes_with_nul().into() }
    }
}

impl<'a> From<CStrPtr<'a>> for CString {
    #[inline]
    fn from(s: CStrPtr<'a>) -> Self {
        s.to_c_str().into()
    }
}

impl AsRef<CStr> for CString {
    #[inline]
    fn as_ref(&self) -> &CStr {
        self.as_c_str()
    }
}

impl alloc::borrow::ToOwned for CStr {
    type Owned = CString;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        CString { inner: self.to_bytes_with_nul().into() }
    }
}

impl alloc::borrow::Borrow<CStr> for CString {
    #[inline]
    fn borrow(&self) -> &CStr {
        self.as_c_str()
    }
}
