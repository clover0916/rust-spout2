use std::{
    ffi::{c_void, CStr, CString},
    marker::PhantomData,
    ptr::NonNull,
    rc::Rc,
};

extern "C" {
    fn rust_spout_dx_create(device: *mut c_void, name: *const std::ffi::c_char) -> *mut c_void;
    fn rust_spout_dx_send(handle: *mut c_void, texture: *mut c_void) -> bool;
    fn rust_spout_dx_release_sender(handle: *mut c_void, name: *const std::ffi::c_char);
    fn rust_spout_dx_destroy(handle: *mut c_void);
}

/// A D3D11 Spout sender using GPU texture copies, without an OpenGL context.
///
/// Owns an additional COM reference to the supplied device. The sender is
/// thread-affine (`!Send`, `!Sync`) and releases its registration on drop.
/// Names use Spout's ANSI encoding; portable ASCII names are recommended.
pub struct DirectXSender {
    raw: NonNull<c_void>,
    name: CString,
    _not_send_sync: PhantomData<Rc<()>>,
}

impl DirectXSender {
    /// Open a sender on an existing D3D11 device. Registration occurs on the
    /// first successful texture submission. Empty/overlong names are rejected.
    ///
    /// # Safety
    /// `device` must be null or a live `ID3D11Device*` for this call. The device
    /// and its immediate context must be usable from the calling thread. If
    /// shared with other threads, the caller must enable D3D multithread
    /// protection or serialize all immediate-context access externally.
    pub unsafe fn new(device: *mut c_void, name: &CStr) -> Option<Self> {
        if device.is_null() || name.to_bytes().is_empty() || name.to_bytes().len() >= 256 {
            return None;
        }
        NonNull::new(unsafe { rust_spout_dx_create(device, name.as_ptr()) }).map(|raw| Self {
            raw,
            name: name.to_owned(),
            _not_send_sync: PhantomData,
        })
    }

    /// Copy a BGRA8/RGBA8 UNORM texture to the Spout shared texture. Resolution
    /// and format changes recreate the shared output automatically. Unsupported
    /// shapes/formats and textures from a different device return false.
    ///
    /// Spout may skip a frame when a receiver owns the access mutex; true means
    /// the submission was accepted, not a guarantee of receiver presentation.
    ///
    /// # Safety
    /// `texture` must be null or a live `ID3D11Texture2D*` until this call returns.
    /// It must be ready for copying and, if keyed, its mutex must be owned by the
    /// caller. The device/context synchronization contract of `new` still applies.
    pub unsafe fn send_texture(&mut self, texture: *mut c_void) -> bool {
        unsafe { rust_spout_dx_send(self.raw.as_ptr(), texture) }
    }

    /// Remove this sender from discovery. A later submission re-registers it.
    pub fn release_sender(&mut self) {
        unsafe { rust_spout_dx_release_sender(self.raw.as_ptr(), self.name.as_ptr()) };
    }
}

impl Drop for DirectXSender {
    fn drop(&mut self) {
        unsafe { rust_spout_dx_destroy(self.raw.as_ptr()) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn null_device_is_rejected() {
        let name = std::ffi::CString::new("test").unwrap();
        assert!(unsafe { DirectXSender::new(std::ptr::null_mut(), &name) }.is_none());
    }
}
