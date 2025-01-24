/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use windows::Win32::Graphics::DirectWrite::IDWriteFontFamily;

use super::*;
use helpers::*;

pub struct FontFamily {
    native: IDWriteFontFamily,
}

impl FontFamily {
    pub fn take(native: IDWriteFontFamily) -> FontFamily {
        FontFamily { native }
    }

    pub fn name(&self) -> String {
        unsafe {
            let family_names = self.native.GetFamilyNames().unwrap();
            get_locale_string(&family_names)
        }
    }

    pub fn get_first_matching_font(
        &self,
        weight: FontWeight,
        stretch: FontStretch,
        style: FontStyle,
    ) -> Font {
        unsafe {
            let font = self
                .native
                .GetFirstMatchingFont(weight.into(), stretch.into(), style.into())
                .unwrap();
            Font::take(font)
        }
    }

    pub fn get_font_collection(&self) -> FontCollection {
        unsafe {
            let collection = self.native.GetFontCollection().unwrap();
            FontCollection::take(collection)
        }
    }

    pub fn get_font_count(&self) -> u32 {
        unsafe { self.native.GetFontCount() }
    }

    pub fn get_font(&self, index: u32) -> Font {
        unsafe {
            let font = self.native.GetFont(index).unwrap();
            Font::take(font)
        }
    }
}
