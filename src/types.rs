/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/* this is include!()'d in lib.rs */
use std::mem;
use windows::Win32::Graphics::DirectWrite::{
    DWRITE_FONT_STRETCH, DWRITE_FONT_STYLE, DWRITE_FONT_WEIGHT
};

// mirrors DWRITE_FONT_WEIGHT
#[cfg_attr(feature = "serde_serialization", derive(Deserialize, Serialize))]
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum FontWeight {
    Thin,
    ExtraLight,
    Light,
    SemiLight,
    Regular,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
    ExtraBlack,
    Unknown(u32)
}

impl FontWeight {
    pub fn to_u32(&self) -> u32 {
        match self {
            FontWeight::Thin=> 100,
            FontWeight::ExtraLight=> 200,
            FontWeight::Light=> 300,
            FontWeight::SemiLight=> 350,
            FontWeight::Regular=> 400,
            FontWeight::Medium=> 500,
            FontWeight::SemiBold=> 600,
            FontWeight::Bold=> 700,
            FontWeight::ExtraBold=> 800,
            FontWeight::Black=> 900,
            FontWeight::ExtraBlack=> 950,
            FontWeight::Unknown(v) => { *v }
        }
    }
    pub fn from_u32(v: u32) -> FontWeight {
        match v {
                100 => FontWeight::Thin,
                200 => FontWeight::ExtraLight,
                300 => FontWeight::Light,
                350 => FontWeight::SemiLight,
                400 => FontWeight::Regular,
                500 => FontWeight::Medium,
                600 => FontWeight::SemiBold,
                700 => FontWeight::Bold,
                800 => FontWeight::ExtraBold,
                900 => FontWeight::Black,
                950 => FontWeight::ExtraBlack,
                _ => FontWeight::Unknown(v)
            }
    }
}

impl From<FontWeight> for DWRITE_FONT_WEIGHT {
    #[inline]
    fn from(value: FontWeight) -> Self {
        DWRITE_FONT_WEIGHT(value.to_u32() as i32)
    }
}
impl From<DWRITE_FONT_WEIGHT> for FontWeight {
    #[inline]
    fn from(value: DWRITE_FONT_WEIGHT) -> FontWeight {
        FontWeight::from_u32(value.0 as u32)
    }
}

// mirrors DWRITE_FONT_STRETCH
#[repr(u32)]
#[cfg_attr(feature = "serde_serialization", derive(Deserialize, Serialize))]
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum FontStretch {
    Undefined = 0,
    UltraCondensed = 1,
    ExtraCondensed = 2,
    Condensed = 3,
    SemiCondensed = 4,
    Normal = 5,
    SemiExpanded = 6,
    Expanded = 7,
    ExtraExpanded = 8,
    UltraExpanded = 9,
}

impl FontStretch {
    pub fn to_u32(&self) -> u32 { unsafe { mem::transmute::<FontStretch, u32>(*self) } }
    pub fn from_u32(v: u32) -> FontStretch { unsafe { mem::transmute::<u32, FontStretch>(v) } }
}

impl From<FontStretch> for DWRITE_FONT_STRETCH {
    #[inline]
    fn from(value: FontStretch) -> Self {
        DWRITE_FONT_STRETCH(value.to_u32() as i32)
    }
}
impl From<DWRITE_FONT_STRETCH> for FontStretch {
    #[inline]
    fn from(value: DWRITE_FONT_STRETCH) -> FontStretch {
        FontStretch::from_u32(value.0 as u32)
    }
}

// mirrors DWRITE_FONT_STYLE
#[repr(u32)]
#[cfg_attr(feature = "serde_serialization", derive(Deserialize, Serialize))]
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum FontStyle {
    Normal = 0,
    Oblique = 1,
    Italic = 2,
}

impl FontStyle {
    pub fn to_u32(&self) -> u32 { unsafe { mem::transmute::<FontStyle, u32>(*self) } }
    pub fn from_u32(v: u32) -> FontStyle { unsafe { mem::transmute::<u32, FontStyle>(v) } }
}

impl From<FontStyle> for DWRITE_FONT_STYLE {
    #[inline]
    fn from(value: FontStyle) -> Self {
        DWRITE_FONT_STYLE(value.to_u32() as i32)
    }
}
impl From<DWRITE_FONT_STYLE> for FontStyle {
    #[inline]
    fn from(value: DWRITE_FONT_STYLE) -> FontStyle {
        FontStyle::from_u32(value.0 as u32)
    }
}

// mirrors DWRITE_FONT_SIMULATIONS
#[repr(u32)]
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum FontSimulations {
    None = windows::Win32::Graphics::DirectWrite::DWRITE_FONT_SIMULATIONS_NONE.0 as u32,
    Bold = windows::Win32::Graphics::DirectWrite::DWRITE_FONT_SIMULATIONS_BOLD.0 as u32,
    Oblique = windows::Win32::Graphics::DirectWrite::DWRITE_FONT_SIMULATIONS_OBLIQUE.0 as u32,
    BoldOblique = (windows::Win32::Graphics::DirectWrite::DWRITE_FONT_SIMULATIONS_BOLD.0 |
        windows::Win32::Graphics::DirectWrite::DWRITE_FONT_SIMULATIONS_OBLIQUE.0) as u32,
}

impl From<windows::Win32::Graphics::DirectWrite::DWRITE_FONT_SIMULATIONS> for FontSimulations {
    #[inline]
    fn from(value: windows::Win32::Graphics::DirectWrite::DWRITE_FONT_SIMULATIONS) -> Self {
        match value.0 {
            x if x == FontSimulations::None as i32 => FontSimulations::None,
            x if x == FontSimulations::Bold as i32 => FontSimulations::Bold,
            x if x == FontSimulations::Oblique as i32 => FontSimulations::Oblique,
            x if x == FontSimulations::BoldOblique as i32 => FontSimulations::BoldOblique,
            _ => FontSimulations::None, // default case
        }
    }
}
impl From<FontSimulations> for windows::Win32::Graphics::DirectWrite::DWRITE_FONT_SIMULATIONS {
    #[inline]
    fn from(value: FontSimulations) -> Self {
        windows::Win32::Graphics::DirectWrite::DWRITE_FONT_SIMULATIONS(value as i32)
    }
}

#[cfg_attr(feature = "serde_serialization", derive(Deserialize, Serialize))]
#[derive(PartialEq, Debug, Clone)]
pub struct FontDescriptor {
    pub family_name: String,
    pub weight: FontWeight,
    pub stretch: FontStretch,
    pub style: FontStyle,
}
