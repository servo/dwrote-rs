/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::ptr;
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory, IDWriteGdiInterop, DWRITE_FACTORY_TYPE_SHARED,
};
use windows::Win32::Graphics::Gdi::HDC;

use super::BitmapRenderTarget;

pub struct GdiInterop {
    native: IDWriteGdiInterop,
}

impl GdiInterop {
    pub fn create() -> GdiInterop {
        unsafe {
            let factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).unwrap();
            let native = factory.GetGdiInterop().unwrap();
            GdiInterop::take(native)
        }
    }

    pub fn take(native: IDWriteGdiInterop) -> GdiInterop {
        GdiInterop { native }
    }

    pub fn create_bitmap_render_target(&self, width: u32, height: u32) -> BitmapRenderTarget {
        unsafe {
            let native = self
                .native
                .CreateBitmapRenderTarget(HDC(ptr::null_mut()), width, height)
                .unwrap();
            BitmapRenderTarget::take(native)
        }
    }
}
