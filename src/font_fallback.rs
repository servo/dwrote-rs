/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use windows::core::HSTRING;
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory2, IDWriteFontFallback, DWRITE_FACTORY_TYPE_SHARED,
};

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
            let factory: IDWriteFactory2 = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).ok()?;
            let native = factory.GetSystemFontFallback().ok()?;
            Some(Self::take(native))
        }
    }

    pub fn take(native: IDWriteFontFallback) -> FontFallback {
        FontFallback { native }
    }

    // TODO: map_characters (main function)
    pub fn map_characters(
        &self,
        text_analysis_source: &TextAnalysisSource,
        text_position: u32,
        text_length: u32,
        base_font: &FontCollection,
        base_family: &str,
        base_weight: FontWeight,
        base_style: FontStyle,
        base_stretch: FontStretch,
    ) -> FallbackResult {
        unsafe {
            let mut font = None;
            let mut mapped_length = 0;
            let mut scale = 0.0;
            self.native
                .MapCharacters(
                    &text_analysis_source.native,
                    text_position,
                    text_length,
                    &base_font.native,
                    &HSTRING::from(base_family),
                    base_weight.into(),
                    base_style.into(),
                    base_stretch.into(),
                    &mut mapped_length,
                    &mut font,
                    &mut scale,
                )
                .unwrap();
            let mapped_font = font.map(|f| Font::take(f));
            FallbackResult {
                mapped_length: mapped_length as usize,
                mapped_font,
                scale,
            }
        }
    }
}
