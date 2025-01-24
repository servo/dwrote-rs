/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use windows::core::Interface;
use windows::Win32::Foundation::{FALSE, TRUE};
use windows::Win32::Graphics::DirectWrite::{
    IDWriteFont, IDWriteFont1, DWRITE_INFORMATIONAL_STRING_ID,
};

use super::*;
use helpers::*;

pub struct Font {
    native: IDWriteFont,
}

impl Font {
    pub fn take(native: IDWriteFont) -> Font {
        Font { native }
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
        unsafe { mem::transmute::<DWRITE_FONT_STRETCH, FontStretch>(self.native.GetStretch()) }
    }

    pub fn style(&self) -> FontStyle {
        unsafe { mem::transmute::<DWRITE_FONT_STYLE, FontStyle>(self.native.GetStyle()) }
    }

    pub fn weight(&self) -> FontWeight {
        FontWeight::from(unsafe { self.native.GetWeight() })
    }

    pub fn is_monospace(&self) -> Option<bool> {
        unsafe {
            self.native
                .cast::<IDWriteFont1>()
                .ok()
                .map(|font_1| font_1.IsMonospacedFont() == TRUE)
        }
    }

    pub fn simulations(&self) -> FontSimulations {
        FontSimulations::from(unsafe { self.native.GetSimulations() })
    }

    pub fn family_name(&self) -> String {
        unsafe {
            let family = self.native.GetFontFamily().unwrap();
            FontFamily::take(family).name()
        }
    }

    pub fn face_name(&self) -> String {
        unsafe {
            let names = self.native.GetFaceNames().unwrap();
            get_locale_string(&names)
        }
    }

    pub fn informational_string(&self, id: InformationalStringId) -> Option<String> {
        unsafe {
            let mut names = None;
            let mut exists = FALSE;
            self.native
                .GetInformationalStrings(id.into(), &mut names, &mut exists)
                .ok()?;
            if exists == TRUE {
                Some(names.map(|names| get_locale_string(&names))?)
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
        unsafe {
            let font_1: Option<IDWriteFont1> = self.native.cast().ok();
            match font_1 {
                None => {
                    let mut metrics = std::mem::zeroed();
                    self.native.GetMetrics(&mut metrics);
                    FontMetrics::Metrics0(metrics)
                }
                Some(font_1) => {
                    let mut metrics_1 = std::mem::zeroed();
                    font_1.GetMetrics(&mut metrics_1);
                    FontMetrics::Metrics1(metrics_1)
                }
            }
        }
    }
}

impl Clone for Font {
    fn clone(&self) -> Font {
        Font {
            native: self.native.clone(),
        }
    }
}

macro_rules! make_const_informational_string {
    ($($name:ident),*) => {
        $(
            const $name: i32 = windows::Win32::Graphics::DirectWrite::$name.0;
        )*
    };
}
make_const_informational_string!(
    DWRITE_INFORMATIONAL_STRING_FULL_NAME,
    DWRITE_INFORMATIONAL_STRING_POSTSCRIPT_NAME,
    DWRITE_INFORMATIONAL_STRING_POSTSCRIPT_CID_NAME,
    DWRITE_INFORMATIONAL_STRING_COPYRIGHT_NOTICE,
    DWRITE_INFORMATIONAL_STRING_DESCRIPTION,
    DWRITE_INFORMATIONAL_STRING_DESIGNER,
    DWRITE_INFORMATIONAL_STRING_DESIGNER_URL,
    DWRITE_INFORMATIONAL_STRING_DESIGN_SCRIPT_LANGUAGE_TAG,
    DWRITE_INFORMATIONAL_STRING_FONT_VENDOR_URL,
    DWRITE_INFORMATIONAL_STRING_LICENSE_DESCRIPTION,
    DWRITE_INFORMATIONAL_STRING_LICENSE_INFO_URL,
    DWRITE_INFORMATIONAL_STRING_MANUFACTURER,
    DWRITE_INFORMATIONAL_STRING_PREFERRED_FAMILY_NAMES,
    DWRITE_INFORMATIONAL_STRING_PREFERRED_SUBFAMILY_NAMES,
    DWRITE_INFORMATIONAL_STRING_SAMPLE_TEXT,
    DWRITE_INFORMATIONAL_STRING_SUPPORTED_SCRIPT_LANGUAGE_TAG,
    DWRITE_INFORMATIONAL_STRING_TRADEMARK,
    DWRITE_INFORMATIONAL_STRING_VERSION_STRINGS,
    DWRITE_INFORMATIONAL_STRING_WIN32_FAMILY_NAMES,
    DWRITE_INFORMATIONAL_STRING_WIN32_SUBFAMILY_NAMES,
    DWRITE_INFORMATIONAL_STRING_WWS_FAMILY_NAME
);

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InformationalStringId {
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
impl From<DWRITE_INFORMATIONAL_STRING_ID> for InformationalStringId {
    fn from(v: DWRITE_INFORMATIONAL_STRING_ID) -> Self {
        match v.0 {
            DWRITE_INFORMATIONAL_STRING_FULL_NAME => Self::FullName,
            DWRITE_INFORMATIONAL_STRING_POSTSCRIPT_NAME => Self::PostscriptName,
            DWRITE_INFORMATIONAL_STRING_POSTSCRIPT_CID_NAME => Self::PostscriptCidName,
            DWRITE_INFORMATIONAL_STRING_COPYRIGHT_NOTICE => Self::CopyrightNotice,
            DWRITE_INFORMATIONAL_STRING_DESCRIPTION => Self::Description,
            DWRITE_INFORMATIONAL_STRING_DESIGNER => Self::Designer,
            DWRITE_INFORMATIONAL_STRING_DESIGNER_URL => Self::DesignerUrl,
            DWRITE_INFORMATIONAL_STRING_DESIGN_SCRIPT_LANGUAGE_TAG => Self::DesignScriptLanguageTag,
            DWRITE_INFORMATIONAL_STRING_FONT_VENDOR_URL => Self::VendorUrl,
            DWRITE_INFORMATIONAL_STRING_LICENSE_DESCRIPTION => Self::LicenseDescription,
            DWRITE_INFORMATIONAL_STRING_LICENSE_INFO_URL => Self::LicenseInfoUrl,
            DWRITE_INFORMATIONAL_STRING_MANUFACTURER => Self::Manufacturer,
            DWRITE_INFORMATIONAL_STRING_PREFERRED_FAMILY_NAMES => Self::PreferredFamilyNames,
            DWRITE_INFORMATIONAL_STRING_PREFERRED_SUBFAMILY_NAMES => Self::PreferredSubfamilyNames,
            DWRITE_INFORMATIONAL_STRING_SAMPLE_TEXT => Self::SampleText,
            DWRITE_INFORMATIONAL_STRING_SUPPORTED_SCRIPT_LANGUAGE_TAG => {
                Self::SupportedScriptLanguageTag
            }
            DWRITE_INFORMATIONAL_STRING_TRADEMARK => Self::Trademark,
            DWRITE_INFORMATIONAL_STRING_VERSION_STRINGS => Self::Version,
            DWRITE_INFORMATIONAL_STRING_WIN32_FAMILY_NAMES => Self::Win32FamilyNames,
            DWRITE_INFORMATIONAL_STRING_WIN32_SUBFAMILY_NAMES => Self::Win32SubfamilyNames,
            DWRITE_INFORMATIONAL_STRING_WWS_FAMILY_NAME => Self::WwsFamilyName,
            _ => panic!("Unknown DWRITE_INFORMATIONAL_STRING_ID"),
        }
    }
}
impl Into<i32> for InformationalStringId {
    fn into(self) -> i32 {
        unsafe { mem::transmute::<InformationalStringId, i32>(self) }
    }
}
impl Into<DWRITE_INFORMATIONAL_STRING_ID> for InformationalStringId {
    #[inline]
    fn into(self) -> DWRITE_INFORMATIONAL_STRING_ID {
        DWRITE_INFORMATIONAL_STRING_ID(self.into())
    }
}
impl Into<u32> for InformationalStringId {
    #[inline]
    fn into(self) -> u32 {
        Into::<i32>::into(self) as u32
    }
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
