/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::mem::MaybeUninit;
use windows::Win32::Foundation::{FALSE, TRUE};
use windows::Win32::Graphics::DirectWrite::{IDWriteFont, IDWriteFont1};
use windows_core::Interface;

use super::*;
use helpers::*;

#[derive(Clone)]
pub struct Font {
    native: IDWriteFont,
}

impl Font {
    pub fn take(native: IDWriteFont) -> Font {
        Font { native }
    }

    pub fn as_ptr(&self) -> &IDWriteFont {
        &self.native
    }

    pub fn to_descriptor(&self) -> FontDescriptor {
        FontDescriptor {
            family_name: self.family_name(),
            stretch: self.stretch(),
            style: self.style(),
            weight: self.weight(),
        }
    }

    pub fn stretch(&self) -> FontStretch {
        unsafe { self.native.GetStretch().into() }
    }

    pub fn style(&self) -> FontStyle {
        unsafe { self.native.GetStyle().into() }
    }

    pub fn weight(&self) -> FontWeight {
        unsafe { self.native.GetWeight().into() }
    }

    pub fn is_monospace(&self) -> Option<bool> {
        unsafe {
            let font1 = self.native.cast::<IDWriteFont1>().ok();
            font1.map(|font| font.IsMonospacedFont() == TRUE)
        }
    }

    pub fn simulations(&self) -> FontSimulations {
        unsafe { self.native.GetSimulations().into() }
    }

    pub fn family_name(&self) -> String {
        let family = unsafe { self.native.GetFontFamily().unwrap() };
        let family = FontFamily::take(family);
        family.family_name().unwrap()
    }

    pub fn face_name(&self) -> String {
        let faces = unsafe { self.native.GetFaceNames().unwrap() };
        get_locale_string(faces).unwrap()
    }

    pub fn informational_string(&self, id: InformationalStringId) -> Option<String> {
        unsafe {
            let mut exists = FALSE;
            let mut strings = None;
            self
                .native
                .GetInformationalStrings(id.into(), &mut strings, &mut exists)
                .ok()?;
            if exists == TRUE {
                strings.and_then(|s| get_locale_string(s).ok())
            } else {
                None
            }
        }
    }

    pub fn create_font_face(&self) -> FontFace {
        // FIXME create_font_face should cache the FontFace and return it,
        // there's a 1:1 relationship
        unsafe {
            let face = self.native.CreateFontFace().unwrap();
            FontFace::take(face)
        }
    }

    pub fn metrics(&self) -> FontMetrics {
        let font1 = self.native.cast::<IDWriteFont1>().ok();
        match font1 {
            None => unsafe {
                let mut metrics = MaybeUninit::uninit();
                self.native.GetMetrics(metrics.as_mut_ptr());
                FontMetrics::Metrics0(metrics.assume_init())
            }
            Some(font1) => unsafe {
                let mut metrics1 = MaybeUninit::uninit();
                font1.GetMetrics(metrics1.as_mut_ptr());
                FontMetrics::Metrics1(metrics1.assume_init())
            }
        }
    }
}

macro_rules! define_informational_string_id {
    ( $($name:ident = $value:ident),* $(,)? ) => {
        #[repr(u32)]
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum InformationalStringId {
            $( $name = ::windows::Win32::Graphics::DirectWrite::$value.0 as u32, )*
        }

        impl From<InformationalStringId> for windows::Win32::Graphics::DirectWrite::DWRITE_INFORMATIONAL_STRING_ID {
            fn from(id: InformationalStringId) -> Self {
                match id {
                    $( InformationalStringId::$name => windows::Win32::Graphics::DirectWrite::$value, )*
                }
            }
        }
        impl From<windows::Win32::Graphics::DirectWrite::DWRITE_INFORMATIONAL_STRING_ID> for InformationalStringId {
            fn from(id: windows::Win32::Graphics::DirectWrite::DWRITE_INFORMATIONAL_STRING_ID) -> Self {
                match id {
                    $( windows::Win32::Graphics::DirectWrite::$value => InformationalStringId::$name, )*
                    _ => panic!("Unknown InformationalStringId value"),
                }
            }
        }
    };
}
define_informational_string_id! {
    FullName = DWRITE_INFORMATIONAL_STRING_FULL_NAME,
    PostscriptName = DWRITE_INFORMATIONAL_STRING_POSTSCRIPT_NAME,
    PostscriptCidName = DWRITE_INFORMATIONAL_STRING_POSTSCRIPT_CID_NAME,
    CopyrightNotice = DWRITE_INFORMATIONAL_STRING_COPYRIGHT_NOTICE,
    Description = DWRITE_INFORMATIONAL_STRING_DESCRIPTION,
    Designer = DWRITE_INFORMATIONAL_STRING_DESIGNER,
    DesignerUrl = DWRITE_INFORMATIONAL_STRING_DESIGNER_URL,
    DesignScriptLanguageTag = DWRITE_INFORMATIONAL_STRING_DESIGN_SCRIPT_LANGUAGE_TAG,
    VendorUrl = DWRITE_INFORMATIONAL_STRING_FONT_VENDOR_URL,
    LicenseDescription = DWRITE_INFORMATIONAL_STRING_LICENSE_DESCRIPTION,
    LicenseInfoUrl = DWRITE_INFORMATIONAL_STRING_LICENSE_INFO_URL,
    Manufacturer = DWRITE_INFORMATIONAL_STRING_MANUFACTURER,
    PreferredFamilyNames = DWRITE_INFORMATIONAL_STRING_PREFERRED_FAMILY_NAMES,
    PreferredSubfamilyNames = DWRITE_INFORMATIONAL_STRING_PREFERRED_SUBFAMILY_NAMES,
    SampleText = DWRITE_INFORMATIONAL_STRING_SAMPLE_TEXT,
    SupportedScriptLanguageTag = DWRITE_INFORMATIONAL_STRING_SUPPORTED_SCRIPT_LANGUAGE_TAG,
    Trademark = DWRITE_INFORMATIONAL_STRING_TRADEMARK,
    Version = DWRITE_INFORMATIONAL_STRING_VERSION_STRINGS,
    Win32FamilyNames = DWRITE_INFORMATIONAL_STRING_WIN32_FAMILY_NAMES,
    Win32SubfamilyNames = DWRITE_INFORMATIONAL_STRING_WIN32_SUBFAMILY_NAMES,
    WwsFamilyName = DWRITE_INFORMATIONAL_STRING_WWS_FAMILY_NAME,
}
/// A wrapper around the `DWRITE_FONT_METRICS` and `DWRITE_FONT_METRICS1` types.
pub enum FontMetrics {
    /// Windows 7.
    Metrics0(FontMetrics0),
    /// Windows 8 and up.
    Metrics1(FontMetrics1),
}

impl FontMetrics {
    /// Convert self to the Metrics0 arm (throwing away additional information)
    #[inline]
    pub fn metrics0(self) -> FontMetrics0 {
        match self {
            FontMetrics::Metrics0(metrics) => metrics,
            FontMetrics::Metrics1(metrics) => metrics.Base,
        }
    }
}
