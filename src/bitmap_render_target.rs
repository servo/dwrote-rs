/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::mem::{size_of, zeroed, ManuallyDrop};
use std::slice;
use windows::Win32::Foundation::{COLORREF, FALSE, RECT};
use windows::Win32::Graphics::DirectWrite::{
    IDWriteBitmapRenderTarget, DWRITE_GLYPH_OFFSET, DWRITE_GLYPH_RUN, DWRITE_MEASURING_MODE,
};
use windows::Win32::Graphics::Gdi::{GetCurrentObject, GetObjectW, BITMAP, HDC, OBJ_BITMAP};

use super::{FontFace, RenderingParams};

pub struct BitmapRenderTarget {
    native: IDWriteBitmapRenderTarget,
}

impl BitmapRenderTarget {
    pub fn take(native: IDWriteBitmapRenderTarget) -> BitmapRenderTarget {
        BitmapRenderTarget { native }
    }

    // A dip is 1/96th of an inch, so this value is the number of pixels per inch divided by 96.
    pub fn set_pixels_per_dip(&self, ppd: f32) {
        unsafe {
            let _ = self.native.SetPixelsPerDip(ppd);
        }
    }

    pub fn get_memory_dc(&self) -> HDC {
        unsafe { self.native.GetMemoryDC() }
    }

    pub fn draw_glyph_run(
        &self,
        baseline_origin_x: f32,
        baseline_origin_y: f32,
        measuring_mode: DWRITE_MEASURING_MODE,
        font_face: &FontFace,
        em_size: f32,
        glyph_indices: &[u16],
        glyph_advances: &[f32],
        glyph_offsets: &[DWRITE_GLYPH_OFFSET],
        rendering_params: &RenderingParams,
        color: &(f32, f32, f32),
    ) -> RECT {
        unsafe {
            assert!(glyph_indices.len() == glyph_advances.len());
            assert!(glyph_indices.len() == glyph_offsets.len());

            let r = (color.0 * 255.0) as u8;
            let g = (color.1 * 255.0) as u8;
            let b = (color.2 * 255.0) as u8;
            let color = COLORREF((r as u32) | ((g as u32) << 8) | ((b as u32) << 16));

            let glyph_run = DWRITE_GLYPH_RUN {
                fontFace: ManuallyDrop::new(Some(font_face.native.clone())),
                fontEmSize: em_size,
                glyphCount: glyph_indices.len() as u32,
                glyphIndices: glyph_indices.as_ptr(),
                glyphAdvances: glyph_advances.as_ptr(),
                glyphOffsets: glyph_offsets.as_ptr(),
                isSideways: FALSE,
                bidiLevel: 0,
            };

            let mut rect: RECT = zeroed();
            self.native
                .DrawGlyphRun(
                    baseline_origin_x,
                    baseline_origin_y,
                    measuring_mode,
                    &glyph_run,
                    &rendering_params.native,
                    color,
                    Some(&mut rect),
                )
                .unwrap();
            rect
        }
    }

    // This function expects to have glyphs rendered in WHITE,
    // and pulls out a u8 vector of width*height*4 size with
    // the coverage value (we pull out R) broadcast to the alpha
    // channel, with the color white.  That is, it performs:
    // RGBX -> xxxR, where xxx = 0xff
    pub fn get_opaque_values_as_mask(&self) -> Vec<u8> {
        // Now grossness to pull out the pixels
        unsafe {
            let memory_dc = self.get_memory_dc();
            let mut bitmap: BITMAP = zeroed();
            let ret = GetObjectW(
                GetCurrentObject(HDC(memory_dc.0), OBJ_BITMAP),
                size_of::<BITMAP>() as i32,
                Some(&mut bitmap as *mut _ as *mut _),
            );
            assert!(ret == size_of::<BITMAP>() as i32);
            assert!(bitmap.bmBitsPixel == 32);

            let width = bitmap.bmWidth as usize;
            let stride = bitmap.bmWidthBytes as usize;
            let height = bitmap.bmHeight as usize;

            let mut out_bytes: Vec<u8> = vec![0; width * height * 4];
            let out_u32 =
                slice::from_raw_parts_mut(out_bytes.as_mut_ptr() as *mut u32, width * height);

            for row in 0..height {
                let in_offset = (row * stride) as isize;
                let in_u32 =
                    slice::from_raw_parts(bitmap.bmBits.offset(in_offset) as *const u32, width);
                for col in 0..width {
                    let r = in_u32[col] & 0xff;
                    out_u32[width * row + col] = (r << 24) | (0x00ffffffu32);
                }
            }

            out_bytes
        }
    }
}
