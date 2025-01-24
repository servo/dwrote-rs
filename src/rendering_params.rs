/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory, IDWriteRenderingParams, DWRITE_FACTORY_TYPE_SHARED,
};

pub struct RenderingParams {
    pub(crate) native: IDWriteRenderingParams,
}

impl RenderingParams {
    pub fn create_for_primary_monitor() -> RenderingParams {
        unsafe {
            let factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).unwrap();
            let native = factory.CreateRenderingParams().unwrap();
            RenderingParams::take(native)
        }
    }

    pub fn take(native: IDWriteRenderingParams) -> RenderingParams {
        RenderingParams { native }
    }
}
