use core::fmt::{Display, Formatter, Result};
use core::{mem, str};

pub trait MemCast {
    fn cast_ref<T>(&self) -> &T;
    fn cast_mut<T>(&mut self) -> &mut T;
}

impl MemCast for [u8] {
    fn cast_ref<T>(&self) -> &T {
        assert!(self.len() >= mem::size_of::<T>());
        unsafe { &*self.as_ptr().cast::<T>() }
    }

    fn cast_mut<T>(&mut self) -> &mut T {
        assert!(self.len() >= mem::size_of::<T>());
        unsafe { &mut *(self.as_ptr().cast::<T>() as *mut T) }
    }
}

pub struct BytesDisplay<'a>(&'a [u8]);

impl Display for BytesDisplay<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if let Ok(s) = str::from_utf8(self.0) {
            write!(f, "\"{}\"", s)
        } else {
            write!(f, "{:?}", self.0)
        }
    }
}

impl<'a> From<&'a [u8]> for BytesDisplay<'a> {
    fn from(v: &'a [u8]) -> Self {
        BytesDisplay(v)
    }
}
