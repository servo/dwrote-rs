/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::UnsafeCell;
use std::ptr;
use winapi::shared::winerror;
use winapi::um::dwrite::IDWriteLocalizedStrings;
use winapi::um::dwrite::{IDWriteFont, IDWriteFontCollection, IDWriteFontFamily};
use wio::com::ComPtr;

use super::*;
use helpers::*;

pub struct FontFamily {
    native: UnsafeCell<ComPtr<IDWriteFontFamily>>,
}

impl FontFamily {
    pub fn take(native: ComPtr<IDWriteFontFamily>) -> FontFamily {
        FontFamily {
            native: UnsafeCell::new(native),
        }
    }

    pub unsafe fn as_ptr(&self) -> *mut IDWriteFontFamily {
        (*self.native.get()).as_raw()
    }

    pub fn name(&self) -> String {
        self.try_name().unwrap()
    }

    pub fn try_name(&self) -> Result<String, HRESULT> {
        let mut family_names: *mut IDWriteLocalizedStrings = ptr::null_mut();
        unsafe {
            let hr = (*self.native.get()).GetFamilyNames(&mut family_names);
            if hr != 0 {
                return Err(winerror::HRESULT_CODE(hr));
            }
            Ok(get_locale_string(&mut ComPtr::from_raw(family_names)))
        }
    }

    pub fn get_first_matching_font(
        &self,
        weight: FontWeight,
        stretch: FontStretch,
        style: FontStyle,
    ) -> Font {
        self.try_get_first_matching_font(weight, stretch, style)
            .unwrap()
    }

    pub fn try_get_first_matching_font(
        &self,
        weight: FontWeight,
        stretch: FontStretch,
        style: FontStyle,
    ) -> Result<Font, HRESULT> {
        let mut font: *mut IDWriteFont = ptr::null_mut();
        unsafe {
            let hr = (*self.native.get()).GetFirstMatchingFont(
                weight.t(),
                stretch.t(),
                style.t(),
                &mut font,
            );
            if hr != 0 {
                return Err(winerror::HRESULT_CODE(hr));
            }
            Ok(Font::take(ComPtr::from_raw(font)))
        }
    }

    pub fn get_font_collection(&self) -> FontCollection {
        self.try_get_font_collection().unwrap()
    }

    pub fn try_get_font_collection(&self) -> Result<FontCollection, HRESULT> {
        let mut collection: *mut IDWriteFontCollection = ptr::null_mut();
        unsafe {
            let hr = (*self.native.get()).GetFontCollection(&mut collection);
            if hr != 0 {
                return Err(winerror::HRESULT_CODE(hr));
            }
            Ok(FontCollection::take(ComPtr::from_raw(collection)))
        }
    }

    pub fn get_font_count(&self) -> u32 {
        unsafe { (*self.native.get()).GetFontCount() }
    }

    pub fn get_font(&self, index: u32) -> Font {
        self.try_get_font(index).unwrap()
    }

    pub fn try_get_font(&self, index: u32) -> Result<Font, HRESULT> {
        let mut font: *mut IDWriteFont = ptr::null_mut();
        unsafe {
            let hr = (*self.native.get()).GetFont(index, &mut font);
            if hr != 0 {
                return Err(winerror::HRESULT_CODE(hr));
            }
            Ok(Font::take(ComPtr::from_raw(font)))
        }
    }
}
