/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use windows::Win32::Graphics::DirectWrite::IDWriteFontFamily;
use windows_core::HRESULT;

use super::helpers::get_locale_string;
use super::*;

#[derive(Debug, Clone)]
pub struct FontFamily {
    native: IDWriteFontFamily,
}

impl FontFamily {
    pub fn take(native: IDWriteFontFamily) -> FontFamily {
        FontFamily {
            native,
        }
    }

    pub unsafe fn as_ptr(&self) -> *mut IDWriteFontFamily {
        // self.native
        unimplemented!()
    }

    #[deprecated(note = "Use `family_name` instead.")]
    pub fn name(&self) -> String {
        self.family_name().unwrap()
    }

    pub fn family_name(&self) -> Result<String, HRESULT> {
        unsafe {
            let family_names = self.native.GetFamilyNames().map_err(|e| e.code())?;
            get_locale_string(family_names).map_err(|e| e.code())
        }
    }

    #[deprecated(note = "Use `first_matching_font` instead.")]
    pub fn get_first_matching_font(
        &self,
        weight: FontWeight,
        stretch: FontStretch,
        style: FontStyle,
    ) -> Font {
        self.first_matching_font(weight, stretch, style).unwrap()
    }

    pub fn first_matching_font(
        &self,
        weight: FontWeight,
        stretch: FontStretch,
        style: FontStyle,
    ) -> Result<Font, HRESULT> {
        unsafe {
            let font = self.native.GetFirstMatchingFont(
                weight.into(),
                stretch.into(),
                style.into(),
            ).map_err(|err| err.code())?;
            Ok(Font::take(font))
        }
    }

    #[deprecated(note = "Use `font_collection` instead.")]
    pub fn get_font_collection(&self) -> FontCollection {
        self.font_collection().unwrap()
    }

    pub fn font_collection(&self) -> Result<FontCollection, HRESULT> {
        unsafe {
            let collection = self.native.GetFontCollection().map_err(|e| e.code())?;
            Ok(FontCollection::take(collection))
        }
    }

    pub fn get_font_count(&self) -> u32 {
         unsafe { self.native.GetFontCount() }
    }

    #[deprecated(note = "Use `font` instead.")]
    pub fn get_font(&self, index: u32) -> Font {
        self.font(index).unwrap()
    }

    pub fn font(&self, index: u32) -> Result<Font, HRESULT> {
        unsafe {
            let font = self.native.GetFont(index).map_err(|e| e.code())?;
            Ok(Font::take(font))
        }
    }
}
