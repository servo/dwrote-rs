/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::DirectWrite::{IDWriteGlyphRunAnalysis, DWRITE_GLYPH_RUN, DWRITE_MATRIX, DWRITE_RENDERING_MODE, DWRITE_MEASURING_MODE, DWRITE_TEXTURE_TYPE, DWRITE_TEXTURE_CLEARTYPE_3x1, DWRITE_TEXTURE_ALIASED_1x1};
use windows_core::HRESULT;

use super::DWriteFactory;

pub struct GlyphRunAnalysis {
    native: IDWriteGlyphRunAnalysis,
}

impl GlyphRunAnalysis {
    pub fn create(
        glyph_run: &DWRITE_GLYPH_RUN,
        pixels_per_dip: f32,
        transform: Option<DWRITE_MATRIX>,
        rendering_mode: DWRITE_RENDERING_MODE,
        measuring_mode: DWRITE_MEASURING_MODE,
        baseline_x: f32,
        baseline_y: f32,
    ) -> Result<GlyphRunAnalysis, HRESULT> {
        unsafe {
            let native = DWriteFactory().CreateGlyphRunAnalysis(
                glyph_run,
                pixels_per_dip,
                transform.map(|t| &t as *const _),
                rendering_mode,
                measuring_mode,
                baseline_x,
                baseline_y,
            ) .map_err(|e| e.code())?;
            Ok(GlyphRunAnalysis::take(native))
        }
    }

    pub fn take(native: IDWriteGlyphRunAnalysis) -> GlyphRunAnalysis {
        GlyphRunAnalysis { native }
    }

    pub fn get_alpha_texture_bounds(
        &self,
        texture_type: DWRITE_TEXTURE_TYPE,
    ) -> Result<RECT, HRESULT> {
        unsafe {
            self.native.GetAlphaTextureBounds(texture_type).map_err(|e| e.code())
        }
    }

    pub fn create_alpha_texture(
        &self,
        texture_type: DWRITE_TEXTURE_TYPE,
        rect: RECT,
    ) -> Result<Vec<u8>, HRESULT> {
        unsafe {
            let rect_pixels = (rect.right - rect.left) * (rect.bottom - rect.top);
            let rect_bytes = rect_pixels
                * match texture_type {
                    DWRITE_TEXTURE_ALIASED_1x1 => 1,
                    DWRITE_TEXTURE_CLEARTYPE_3x1 => 3,
                    _ => panic!("bad texture type specified"),
                };

            let mut out_bytes: Vec<u8> = vec![0; rect_bytes as usize];
            self.native.CreateAlphaTexture(
                texture_type,
                &rect,
                &mut out_bytes,
            ).map_err(|e| e.code())?;
            Ok(out_bytes)
        }
    }
}
