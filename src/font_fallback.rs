/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::mem::MaybeUninit;

use windows::Win32::Graphics::DirectWrite::{IDWriteFactory2, IDWriteFontFallback};
use windows_core::{Interface, PCWSTR};

use crate::helpers::ToWide;

use super::*;

pub struct FontFallback {
    native: IDWriteFontFallback,
}

pub struct FallbackResult {
    /// Length of mapped substring, in utf-16 code units.
    pub mapped_length: usize,
    /// The font that should be used to render the substring.
    pub mapped_font: Option<Font>,
    /// The scale factor to apply.
    pub scale: f32,
}

impl FontFallback {
    pub fn get_system_fallback() -> Option<FontFallback> {
        unsafe {
            let factory = DWriteFactory();
            let factory2 = factory.cast::<IDWriteFactory2>().ok();
            let factory2 = factory2?;
            let native = factory2.GetSystemFontFallback().ok()?;
            Some(FontFallback::take(native))
        }
    }

    pub fn take(native: IDWriteFontFallback) -> FontFallback {
        FontFallback { native }
    }

    // TODO: I'm following crate conventions for unsafe, but it's bullshit
    pub fn as_ptr(&self) -> &IDWriteFontFallback {
        &self.native
    }

    // TODO: map_characters (main function)
    pub fn map_characters(
        &self,
        text_analysis_source: TextAnalysisSource,
        text_position: u32,
        text_length: u32,
        base_font: &FontCollection,
        base_family: &str,
        base_weight: FontWeight,
        base_style: FontStyle,
        base_stretch: FontStretch,
    ) -> FallbackResult {
        unsafe {
            let mut mapped_length = 0;
            let mut mapped_font = MaybeUninit::uninit();
            let mut scale = 0.0;
            self.native.MapCharacters(
                text_analysis_source.as_ptr(),
                text_position,
                text_length,
                base_font.as_ptr(),
                PCWSTR(base_family.to_wide_null().as_ptr()),
                base_weight.into(),
                base_style.into(),
                base_stretch.into(),
                &mut mapped_length,
                mapped_font.as_mut_ptr(),
                &mut scale
            ).unwrap();
            let mapped_font = mapped_font.assume_init();
            FallbackResult {
                mapped_length: mapped_length as usize,
                mapped_font: mapped_font.map(|f| Font::take(f)),
                scale,
            }
        }
    }
}
