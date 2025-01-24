/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/* this is include!()'d in lib.rs */
use std::mem;
use std::convert::TryInto;
use windows::Win32::Graphics::DirectWrite::{DWRITE_FONT_STYLE, DWRITE_FONT_WEIGHT, DWRITE_FONT_STRETCH};

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
impl From<DWRITE_FONT_WEIGHT> for FontWeight {
    fn from(v: DWRITE_FONT_WEIGHT) -> FontWeight {
        FontWeight::from_u32(v.0.try_into().unwrap())
    }
}
impl Into<DWRITE_FONT_WEIGHT> for FontWeight {
    fn into(self) -> DWRITE_FONT_WEIGHT {
        DWRITE_FONT_WEIGHT(self.to_u32().try_into().unwrap())
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
impl From<DWRITE_FONT_STRETCH> for FontStretch {
    fn from(v: DWRITE_FONT_STRETCH) -> FontStretch {
        FontStretch::from_u32(v.0.try_into().unwrap())
    }
}
impl Into<DWRITE_FONT_STRETCH> for FontStretch {
    fn into(self) -> DWRITE_FONT_STRETCH {
        DWRITE_FONT_STRETCH(self.to_u32().try_into().unwrap())
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
impl From<DWRITE_FONT_STYLE> for FontStyle {
    fn from(v: DWRITE_FONT_STYLE) -> FontStyle {
        FontStyle::from_u32(v.0.try_into().unwrap())
    }
}
impl Into<DWRITE_FONT_STYLE> for FontStyle {
    fn into(self) -> DWRITE_FONT_STYLE {
        DWRITE_FONT_STYLE(self.to_u32().try_into().unwrap())
    }
}

// mirrors DWRITE_FONT_SIMULATIONS
#[repr(u32)]
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum FontSimulations {
    None = 0b00,
    Bold = 0b01,
    Oblique = 0b10,
    BoldOblique = 0b11,
}
impl From<DWRITE_FONT_SIMULATIONS> for FontSimulations {
    fn from(v: DWRITE_FONT_SIMULATIONS) -> FontSimulations {
        match v.0 {
            0b00 => FontSimulations::None,
            0b01 => FontSimulations::Bold,
            0b10 => FontSimulations::Oblique,
            0b11 => FontSimulations::BoldOblique,
            _ => panic!("Invalid DWRITE_FONT_SIMULATIONS value")
        }
    }
}
impl Into<DWRITE_FONT_SIMULATIONS> for FontSimulations {
    fn into(self) -> DWRITE_FONT_SIMULATIONS {
        match self {
            FontSimulations::None => DWRITE_FONT_SIMULATIONS_NONE,
            FontSimulations::Bold => DWRITE_FONT_SIMULATIONS_BOLD,
            FontSimulations::Oblique => DWRITE_FONT_SIMULATIONS_OBLIQUE,
            FontSimulations::BoldOblique => DWRITE_FONT_SIMULATIONS_BOLD | DWRITE_FONT_SIMULATIONS_OBLIQUE,
        }
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
