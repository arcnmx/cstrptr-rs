use {
    core::{cmp::Ordering, fmt, hint},
    cty::c_char,
};

pub struct CStr {
    inner: [c_char],
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FromBytesWithNulError {
    InteriorNul { position: usize },
    NotNulTerminated,
}

impl FromBytesWithNulError {
    fn interior_nul(position: usize) -> Self {
        FromBytesWithNulError::InteriorNul { position }
    }

    fn not_nul_terminated() -> Self {
        FromBytesWithNulError::NotNulTerminated
    }
}

impl fmt::Display for FromBytesWithNulError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FromBytesWithNulError::InteriorNul { position } => write!(
                f,
                "data provided contains an interior nul byte at byte pos {}",
                position
            ),
            FromBytesWithNulError::NotNulTerminated => f.write_str("data provided is not nul terminated"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for FromBytesWithNulError {}

impl CStr {
    #[inline]
    pub unsafe fn from_ptr<'a>(ptr: *const c_char) -> &'a Self {
        let len = sys::strlen(ptr);
        Self::from_bytes_with_nul_unchecked(core::slice::from_raw_parts(ptr as *const _, len + 1))
    }

    pub fn from_bytes_with_nul(bytes: &[u8]) -> Result<&Self, FromBytesWithNulError> {
        match memchr(0, bytes) {
            Some(nul_pos) =>
                if nul_pos + 1 != bytes.len() {
                    Err(FromBytesWithNulError::interior_nul(nul_pos))
                } else {
                    Ok(unsafe { Self::from_bytes_with_nul_unchecked(bytes) })
                },
            None => Err(FromBytesWithNulError::not_nul_terminated()),
        }
    }

    #[inline]
    #[cfg(feature = "unstable")]
    pub const unsafe fn from_bytes_with_nul_unchecked(bytes: &[u8]) -> &Self {
        &*(bytes as *const _ as *const Self)
    }

    #[inline]
    #[cfg(not(feature = "unstable"))]
    pub unsafe fn from_bytes_with_nul_unchecked(bytes: &[u8]) -> &Self {
        &*(bytes as *const _ as *const Self)
    }

    #[inline]
    pub const fn as_ptr(&self) -> *const c_char {
        self.inner.as_ptr()
    }

    #[inline]
    pub const fn as_c_ptr(&self) -> crate::CStrPtr {
        crate::CStrPtr::from_c_str(self)
    }

    #[inline]
    fn as_bytes(&self) -> &[u8] {
        // TODO: someday make this const?
        let bytes = self.as_bytes_with_nul();
        let len = bytes.len() - 1;
        match bytes.get(..len) {
            Some(bytes) => bytes,
            None => unsafe { hint::unreachable_unchecked() },
        }
    }

    #[inline]
    pub fn to_bytes(&self) -> &[u8] {
        self.as_bytes()
    }

    #[inline]
    #[cfg(feature = "unstable")]
    const fn as_bytes_with_nul(&self) -> &[u8] {
        unsafe { &*(&self.inner as *const _ as *const [u8]) }
    }

    #[inline]
    #[cfg(not(feature = "unstable"))]
    fn as_bytes_with_nul(&self) -> &[u8] {
        unsafe { &*(&self.inner as *const _ as *const [u8]) }
    }

    #[inline]
    #[cfg(not(feature = "unstable"))]
    pub fn to_bytes_with_nul(&self) -> &[u8] {
        self.as_bytes_with_nul()
    }

    #[inline]
    #[cfg(feature = "unstable")]
    pub const fn to_bytes_with_nul(&self) -> &[u8] {
        self.as_bytes_with_nul()
    }

    #[inline]
    pub fn to_str(&self) -> Result<&str, core::str::Utf8Error> {
        core::str::from_utf8(self.as_bytes())
    }

    #[inline]
    #[cfg(feature = "alloc")]
    pub fn to_string_lossy(&self) -> alloc::borrow::Cow<'_, str> {
        alloc::string::String::from_utf8_lossy(self.to_bytes())
    }

    #[cfg(feature = "alloc")]
    #[inline]
    pub fn into_c_string(self: alloc::boxed::Box<Self>) -> crate::CString {
        use alloc::boxed::Box;
        let raw = Box::into_raw(self) as *mut [u8];
        unsafe { crate::CString::from_bytes_with_nul_unchecked(Box::from_raw(raw)) }
    }
}

impl PartialEq for CStr {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.to_bytes().eq(other.to_bytes())
    }
}

impl Eq for CStr {}

impl PartialOrd for CStr {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.to_bytes().partial_cmp(&other.to_bytes())
    }
}

impl Ord for CStr {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_bytes().cmp(&other.to_bytes())
    }
}

impl AsRef<CStr> for CStr {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<'a> Default for &'a CStr {
    fn default() -> Self {
        unsafe { CStr::from_bytes_with_nul_unchecked(&[0]) }
    }
}

impl fmt::Debug for CStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.to_bytes().escape_ascii())
    }
}

#[cfg(feature = "memchr")]
use memchr::memchr;
#[cfg(not(feature = "memchr"))]
fn memchr(n: u8, haystack: &[u8]) -> Option<usize> {
    match unsafe { sys::memchr(haystack.as_ptr() as *const _, n as _, haystack.len()) } {
        ptr if ptr.is_null() => None,
        ptr => Some(ptr as usize - haystack.as_ptr() as usize),
    }
}

mod sys {
    extern "C" {
        pub fn strlen(ptr: *const cty::c_char) -> usize;
        #[cfg(not(feature = "memchr"))]
        pub fn memchr(s: *const cty::c_void, c: cty::c_int, n: usize) -> *mut cty::c_void;
    }
}
