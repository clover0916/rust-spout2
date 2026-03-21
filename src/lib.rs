#![allow(unused_imports)]

#[cfg(not(windows))]
compile_error!("rust-spout2 only supports Windows targets.");

use autocxx::prelude::*;
use std::marker::PhantomData;
use std::pin::Pin;
use std::ptr::NonNull;
use std::rc::Rc;

/// A Windows DWORD.
pub type DWORD = c_ulong;
/// A Windows HANDLE.
pub type HANDLE = *mut c_void;
/// An OpenGL GLuint.
pub type GLuint = c_uint;
/// An OpenGL GLenum.
pub type GLenum = c_uint;

include_cpp! {
    #include "SpoutLibrary.h"

    safety!(unsafe)

    generate!("SPOUTLIBRARY")
}

unsafe extern "system" {
    pub fn GetSpout() -> *mut ffi::SPOUTLIBRARY;
}

/// Safe owner for a `SPOUTLIBRARY*` obtained from `GetSpout`.
///
/// Calls `Release` automatically on drop.
pub struct Spout {
    raw: NonNull<ffi::SPOUTLIBRARY>,
    _not_send_sync: PhantomData<Rc<()>>,
}

impl Spout {
    /// Tries to acquire the Spout library handle.
    pub fn new() -> Option<Self> {
        let raw = unsafe { GetSpout() };
        NonNull::new(raw).map(|raw| Self {
            raw,
            _not_send_sync: PhantomData,
        })
    }

    /// Returns the raw mutable pointer for low-level interop.
    pub fn as_mut_ptr(&self) -> *mut ffi::SPOUTLIBRARY {
        self.raw.as_ptr()
    }

    /// Returns a pinned mutable reference for calling generated methods.
    pub fn as_pin_mut(&mut self) -> Pin<&mut ffi::SPOUTLIBRARY> {
        unsafe { Pin::new_unchecked(self.raw.as_mut()) }
    }
}

impl Drop for Spout {
    fn drop(&mut self) {
        self.as_pin_mut().Release();
    }
}
