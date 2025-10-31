/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use windows::Win32::Graphics::DirectWrite::IDWriteGdiInterop;
use super::{BitmapRenderTarget, DWriteFactory};

pub struct GdiInterop {
    native: IDWriteGdiInterop,
}

impl GdiInterop {
    pub fn create() -> GdiInterop {
        unsafe {
            let native = DWriteFactory().GetGdiInterop().unwrap();
            GdiInterop::take(native)
        }
    }

    pub fn take(native: IDWriteGdiInterop) -> GdiInterop {
        GdiInterop { native }
    }

    pub fn create_bitmap_render_target(&self, width: u32, height: u32) -> BitmapRenderTarget {
        unsafe {
            let native = self.native.CreateBitmapRenderTarget(
                None,
                width,
                height,
            ).unwrap();
            BitmapRenderTarget::take(native)
        }
    }
}
