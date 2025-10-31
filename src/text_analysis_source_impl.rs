// /* This Source Code Form is subject to the terms of the Mozilla Public
//  * License, v. 2.0. If a copy of the MPL was not distributed with this
//  * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// //! A custom implementation of the "text analysis source" interface so that
// //! we can convey data to the `FontFallback::map_characters` method.

#![allow(non_snake_case)]

use std::os::windows::ffi::OsStrExt;
use std::borrow::Cow;
use std::ffi::OsStr;

use libc::wchar_t;
use windows::Win32::Foundation::E_INVALIDARG;
use windows::Win32::Graphics::DirectWrite::{DWRITE_NUMBER_SUBSTITUTION_METHOD, DWRITE_READING_DIRECTION, IDWriteNumberSubstitution, IDWriteTextAnalysisSource, IDWriteTextAnalysisSource_Impl};
use windows_core::{PCWSTR, implement};

use super::helpers::ToWide;
use super::*;

#[implement(IDWriteTextAnalysisSource)]
pub struct CustomTextAnalysisSourceImpl<'a> {
    inner: Box<dyn TextAnalysisSourceMethods + 'a>,
    text: Cow<'a, [wchar_t]>,
    number_subst: Option<NumberSubstitution>,
}

impl<'a> CustomTextAnalysisSourceImpl<'a> {
    /// Create a new custom TextAnalysisSource for the given text and a trait
    /// implementation.
    ///
    /// Note: this method has no NumberSubsitution specified. See
    /// `from_text_and_number_subst` if you need number substitution.
    pub fn from_text(
        inner: Box<dyn TextAnalysisSourceMethods + 'a>,
        text: Cow<'a, [wchar_t]>,
    ) -> CustomTextAnalysisSourceImpl<'a> {
        CustomTextAnalysisSourceImpl { inner, text, number_subst: None }
    }

    /// Create a new custom TextAnalysisSource for the given text and a trait
    /// implementation.
    ///
    /// Note: this method only supports a single `NumberSubstitution` for the
    /// entire string.
    pub fn from_text_and_number_subst(
        inner: Box<dyn TextAnalysisSourceMethods + 'a>,
        text: Cow<'a, [wchar_t]>,
        number_subst: NumberSubstitution,
    ) -> CustomTextAnalysisSourceImpl<'a> {
        CustomTextAnalysisSourceImpl { inner, text, number_subst: Some(number_subst) }
    }
}

impl<'a> IDWriteTextAnalysisSource_Impl for CustomTextAnalysisSourceImpl_Impl<'a> {
    fn GetTextAtPosition(&self, text_position: u32, text_string: *mut *mut u16, text_length: *mut u32) -> windows_core::Result<()> {
        unsafe {
            if text_position >= (self.text.len() as u32) {
                *text_string = std::ptr::null_mut();
                *text_length = 0;
                return Ok(());
            }
            *text_string = self.text.as_ptr().add(text_position as usize) as *mut u16;
            *text_length = (self.text.len() as u32) - text_position;
            Ok(())
        }
    }

    fn GetTextBeforePosition(&self, text_position: u32, text_string: *mut *mut u16, text_length: *mut u32) -> windows_core::Result<()> {
        unsafe {
            if text_position == 0 || text_position > (self.text.len() as u32) {
                *text_string = std::ptr::null_mut();
                *text_length = 0;
                return Ok(());
            }
            *text_string = self.text.as_ptr() as *mut u16;
            *text_length = text_position;
            Ok(())
        }
    }

    fn GetParagraphReadingDirection(&self) -> DWRITE_READING_DIRECTION {
        self.inner.get_paragraph_reading_direction()
    }

    fn GetLocaleName(&self, text_position: u32, text_length: *mut u32, locale_name: *mut *mut u16) -> windows_core::Result<()> {
        unsafe {
            let (locale, text_len) = self.inner.get_locale_name(text_position);

            // Copy the locale data into the buffer
            let mut locale_buf: [u16; 32] = [0; 32];
            for (i, c) in OsStr::new(&*locale).encode_wide().chain(Some(0)).enumerate() {
                // -1 here is deliberate: it ensures that we never write to the last character in
                // this.locale_buf, so that the buffer is always null-terminated.
                if i >= locale_buf.len() - 1 {
                    break;
                }
                *locale_buf.get_unchecked_mut(i) = c;
            }

            *text_length = text_len;
            *locale_name = locale_buf.as_ptr() as *mut u16;
            Ok(())
        }
    }

    fn GetNumberSubstitution(&self, text_position: u32, text_length: *mut u32, number_substitution: windows_core::OutRef<IDWriteNumberSubstitution>) -> windows_core::Result<()> {
        unsafe {
            if text_position >= (self.text.len() as u32) {
                return Err(windows_core::Error::from_hresult(E_INVALIDARG));
            }

            *text_length = (self.text.len() as u32) - text_position;
            number_substitution.write(self.number_subst.as_ref().map(|n| n.native.clone()))?;

            Ok(())
        }
    }
}

/// A wrapped version of an `IDWriteNumberSubstitution` object.
pub struct NumberSubstitution {
    pub(crate) native: IDWriteNumberSubstitution,
}

impl NumberSubstitution {
    pub fn new(
        subst_method: DWRITE_NUMBER_SUBSTITUTION_METHOD,
        locale: &str,
        ignore_user_overrides: bool,
    ) -> NumberSubstitution {
        unsafe {
            let native = DWriteFactory().CreateNumberSubstitution(
                subst_method,
                PCWSTR(locale.to_wide_null().as_ptr()),
                ignore_user_overrides,
            ).expect("error creating number substitution");
            NumberSubstitution { native }
        }
    }
}

/// The Rust side of a custom text analysis source implementation.
pub trait TextAnalysisSourceMethods {
    /// Determine the locale for a range of text.
    ///
    /// Return locale and length of text (in utf-16 code units) for which the
    /// locale is valid.
    fn get_locale_name(&self, text_position: u32) -> (Cow<'_, str>, u32);

    /// Get the text direction for the paragraph.
    fn get_paragraph_reading_direction(&self) -> DWRITE_READING_DIRECTION;
}
