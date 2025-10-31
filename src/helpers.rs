/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

use windows::Win32::Foundation::{E_FAIL, FALSE};
use windows::Win32::Globalization::GetUserDefaultLocaleName;
use windows::Win32::Graphics::DirectWrite::IDWriteLocalizedStrings;
use windows::Win32::System::SystemServices::LOCALE_NAME_MAX_LENGTH;
use windows_core::PCWSTR;

lazy_static! {
    static ref SYSTEM_LOCALE: Vec<u16> = {
        unsafe {
            let mut locale: Vec<u16> = vec![0; LOCALE_NAME_MAX_LENGTH as usize];
            let length = GetUserDefaultLocaleName(&mut locale);
            locale.truncate(length as usize);
            locale
        }
    };
    static ref EN_US_LOCALE: Vec<u16> = OsStr::new("en-us").to_wide_null();
}

pub fn get_locale_string(strings: IDWriteLocalizedStrings) -> windows_core::Result<String> {
    unsafe {
        let mut index: u32 = 0;
        let mut exists = FALSE;
        let mut res = strings.FindLocaleName(PCWSTR(SYSTEM_LOCALE.as_ptr()), &mut index, &mut exists);
        if res.is_err() || exists == FALSE {
            res = strings.FindLocaleName(PCWSTR(EN_US_LOCALE.as_ptr()), &mut index, &mut exists);
            if res.is_err() || exists == FALSE {
                // Ultimately fall back to first locale on list
                index = 0;
            }
        }
        let length = strings.GetStringLength(index).map_err(|e| e.code())? as usize;

        let mut name = vec![0u16; length + 1];
        strings.GetString(index, &mut name).map_err(|e| e.code())?;
        name.set_len(length);

        String::from_utf16(&name).or(Err(E_FAIL.into()))
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
