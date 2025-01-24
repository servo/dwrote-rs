/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows::core::HSTRING;
use windows::Win32::Foundation::{BOOL, FALSE};
use windows::Win32::Globalization::GetUserDefaultLocaleName;
use windows::Win32::Graphics::DirectWrite::IDWriteLocalizedStrings;

lazy_static! {
    static ref EN_US_LOCALE: HSTRING = HSTRING::from("en-us");
    static ref SYSTEM_LOCALE: HSTRING = {
        unsafe {
            let mut locale: Vec<u16> = vec![0; 85];
            let len = GetUserDefaultLocaleName(&mut locale);
            if len <= 1 { // 0 is failure, 1 is empty string
                EN_US_LOCALE.clone()
            } else {
                HSTRING::from_wide(&locale).unwrap_or(EN_US_LOCALE.clone())
            }
        }
    };
}

pub fn get_locale_string(strings: &IDWriteLocalizedStrings) -> String {
    unsafe {
        let mut index: u32 = 0;
        let mut exists: BOOL = FALSE;
        strings
            .FindLocaleName::<&HSTRING>(&SYSTEM_LOCALE, &mut index, &mut exists)
            .unwrap();
        if exists == FALSE {
            strings
                .FindLocaleName::<&HSTRING>(&EN_US_LOCALE, &mut index, &mut exists)
                .unwrap();
            if exists == FALSE {
                // Ultimately fall back to first locale on list
                index = 0;
            }
        }
        let length = strings.GetStringLength(index).unwrap() as usize;

        let mut name: Vec<u16> = vec![0; length + 1];
        strings.GetString(index, &mut name).unwrap();
        name.set_len(length);

        String::from_utf16(&name).ok().unwrap()
    }
}

// ToWide from https://github.com/retep998/wio-rs/blob/master/src/wide.rs

pub trait ToWide {
    fn to_wide_null(&self) -> Vec<u16>;
}

impl<T> ToWide for T
where
    T: AsRef<OsStr>,
{
    fn to_wide_null(&self) -> Vec<u16> {
        self.as_ref().encode_wide().chain(Some(0)).collect()
    }
}
