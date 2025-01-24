/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::sync::atomic::{AtomicU32, Ordering};
use windows::core::HSTRING;
use windows::Win32::Foundation::{BOOL, FALSE};
use windows::Win32::Graphics::DirectWrite::DWriteCreateFactory;
use windows::Win32::Graphics::DirectWrite::IDWriteFactory;
use windows::Win32::Graphics::DirectWrite::IDWriteFontCollection;
use windows::Win32::Graphics::DirectWrite::IDWriteFontCollectionLoader;
use windows::Win32::Graphics::DirectWrite::DWRITE_FACTORY_TYPE_SHARED;

use super::{Font, FontDescriptor, FontFace, FontFamily};

static NEXT_ID: AtomicU32 = AtomicU32::new(0);

pub struct FontCollectionFamilyIterator {
    collection: IDWriteFontCollection,
    curr: u32,
    count: u32,
}

impl Iterator for FontCollectionFamilyIterator {
    type Item = FontFamily;
    fn next(&mut self) -> Option<FontFamily> {
        if self.curr == self.count {
            return None;
        }

        unsafe {
            let family = self.collection.GetFontFamily(self.curr).ok()?;
            self.curr += 1;
            Some(FontFamily::take(family))
        }
    }
}

pub struct FontCollection {
    pub(crate) native: IDWriteFontCollection,
}

impl FontCollection {
    pub fn get_system(update: bool) -> FontCollection {
        unsafe {
            let factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).unwrap();
            let mut collection = None;
            factory
                .GetSystemFontCollection(&mut collection, update)
                .unwrap();

            FontCollection {
                native: collection.unwrap(),
            }
        }
    }

    pub fn system() -> FontCollection {
        FontCollection::get_system(false)
    }

    pub fn take(native: IDWriteFontCollection) -> FontCollection {
        FontCollection { native }
    }

    pub fn from_loader(collection_loader: &IDWriteFontCollectionLoader) -> FontCollection {
        unsafe {
            let factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).unwrap();
            factory
                .RegisterFontCollectionLoader(collection_loader)
                .unwrap();
            let id: *const _ = &NEXT_ID.fetch_add(1, Ordering::SeqCst);
            let collection = factory
                .CreateCustomFontCollection(
                    collection_loader,
                    id as *const _,
                    std::mem::size_of::<u32>() as u32,
                )
                .unwrap();
            FontCollection::take(collection)
        }
    }

    pub fn families_iter(&self) -> FontCollectionFamilyIterator {
        unsafe {
            FontCollectionFamilyIterator {
                collection: self.native.clone(),
                curr: 0,
                count: self.native.GetFontFamilyCount(),
            }
        }
    }

    pub fn get_font_family_count(&self) -> u32 {
        unsafe { self.native.GetFontFamilyCount() }
    }

    pub fn get_font_family(&self, index: u32) -> FontFamily {
        unsafe {
            let family = self.native.GetFontFamily(index).unwrap();
            FontFamily::take(family)
        }
    }

    // Find a font matching the given font descriptor in this
    // font collection.
    pub fn get_font_from_descriptor(&self, desc: &FontDescriptor) -> Option<Font> {
        if let Some(family) = self.get_font_family_by_name(&desc.family_name) {
            let font = family.get_first_matching_font(desc.weight, desc.stretch, desc.style);
            // Exact matches only here
            if font.weight() == desc.weight
                && font.stretch() == desc.stretch
                && font.style() == desc.style
            {
                return Some(font);
            }
        }

        None
    }

    pub fn get_font_from_face(&self, face: &FontFace) -> Option<Font> {
        unsafe {
            let font = self.native.GetFontFromFontFace(&face.native).ok()?;
            Some(Font::take(font))
        }
    }

    pub fn get_font_family_by_name(&self, family_name: &str) -> Option<FontFamily> {
        unsafe {
            let mut index: u32 = 0;
            let mut exists: BOOL = FALSE;
            self.native
                .FindFamilyName(&HSTRING::from(family_name), &mut index, &mut exists)
                .ok()?;
            if exists == FALSE {
                return None;
            }

            let mut family = self.native.GetFontFamily(index).ok()?;

            Some(FontFamily::take(family))
        }
    }
}
