/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::marker::PhantomData;
use winapi::ctypes::wchar_t;
use windows::core::Interface;
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory, IDWriteTextAnalysisSource, DWRITE_FACTORY_TYPE_SHARED,
};

use super::*;

pub struct TextAnalysisSource<'a> {
    pub(crate) native: IDWriteTextAnalysisSource,
    phantom: PhantomData<CustomTextAnalysisSourceImpl<'a>>,
}

impl<'a> TextAnalysisSource<'a> {
    /// Create a new custom TextAnalysisSource for the given text and a trait
    /// implementation.
    ///
    /// Note: this method has no NumberSubsitution specified. See
    /// `from_text_and_number_subst` if you need number substitution.
    pub fn from_text(
        inner: Box<dyn TextAnalysisSourceMethods + 'a>,
        text: Cow<'a, [wchar_t]>,
    ) -> TextAnalysisSource<'a> {
        todo!();
        // unsafe {
        //     let factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).unwrap();
        //     let analyzer = factory.CreateTextAnalyzer().unwrap();

        //     let native = IDWriteTextAnalysisSource::from_raw(
        //         CustomTextAnalysisSourceImpl::from_text_native(inner, text) as *mut _,
        //     );
        //     TextAnalysisSource {
        //         native,
        //         phantom: PhantomData,
        //     }
        // }
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
    ) -> TextAnalysisSource<'a> {
        todo!();
        // let native = unsafe {
        //     CustomTextAnalysisSourceImpl::from_text_and_number_subst_native(
        //         inner,
        //         text,
        //         number_subst,
        //     )
        //     .into_interface()
        // };
        // TextAnalysisSource {
        //     native,
        //     phantom: PhantomData,
        // }
    }
}
