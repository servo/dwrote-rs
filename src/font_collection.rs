/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use windows::Win32::Foundation::FALSE;
use windows::Win32::Graphics::DirectWrite::{IDWriteFontCollection, IDWriteFontCollectionLoader};
use windows_core::{BOOL, HRESULT, PCWSTR};

use super::helpers::ToWide;
use super::{Font, FontDescriptor, FontFace, FontFamily, DWriteFactory};

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
    native: IDWriteFontCollection,
}

impl FontCollection {
    pub fn get_system(update: bool) -> FontCollection {
        unsafe {
            let factory = DWriteFactory();
            let mut fontcollection = MaybeUninit::uninit();
            factory.GetSystemFontCollection(fontcollection.as_mut_ptr(), update).unwrap();
            FontCollection::take(fontcollection.assume_init().unwrap())
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
            let factory = DWriteFactory();
            factory.RegisterFontCollectionLoader(collection_loader).unwrap();
            let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
            let collection = factory.CreateCustomFontCollection(
                collection_loader,
                &id as *const _ as *const _,
                std::mem::size_of::<AtomicUsize>() as u32
            ).unwrap();
            FontCollection::take(collection)
        }
    }

    pub unsafe fn as_ptr(&self) -> &IDWriteFontCollection {
        &self.native
    }

    pub fn families_iter(&self) -> FontCollectionFamilyIterator {
        FontCollectionFamilyIterator {
            collection: self.native.clone(),
            curr: 0,
            count: self.get_font_family_count(),
        }
    }

    pub fn get_font_family_count(&self) -> u32 {
        unsafe { self.native.GetFontFamilyCount() }
    }

    #[deprecated(note = "Use `font_family` instead.")]
    pub fn get_font_family(&self, index: u32) -> FontFamily {
        self.font_family(index).unwrap()
    }

    /// Returns the [`FontFamily`] at the given index.
    pub fn font_family(&self, index: u32) -> Result<FontFamily, HRESULT> {
        unsafe {
            let family = self.native.GetFontFamily(index).map_err(|e| e.code())?;
            Ok(FontFamily::take(family))
        }
    }

    #[deprecated(note = "Use `font_from_descriptor` instead.")]
    pub fn get_font_from_descriptor(&self, desc: &FontDescriptor) -> Option<Font> {
        self.font_from_descriptor(desc).unwrap()
    }

    /// Find a font matching the given font descriptor in this [`FontCollection`].
    pub fn font_from_descriptor(&self, desc: &FontDescriptor) -> Result<Option<Font>, HRESULT> {
        if let Some(family) = self.font_family_by_name(&desc.family_name)? {
            let font = family.first_matching_font(desc.weight, desc.stretch, desc.style)?;
            // Exact matches only here
            if font.weight() == desc.weight
                && font.stretch() == desc.stretch
                && font.style() == desc.style
            {
                return Ok(Some(font));
            }
        }

        Ok(None)
    }

    #[deprecated(note = "Use `font_from_face` instead.")]
    pub fn get_font_from_face(&self, face: &FontFace) -> Option<Font> {
        self.font_from_face(face).ok()
    }

    /// Get a [`Font`] from the given [`FontFace`].
    pub fn font_from_face(&self, face: &FontFace) -> Result<Font, HRESULT> {
        unsafe {
            let font = self.native.GetFontFromFontFace(face.as_ptr()).map_err(|e| e.code())?;
            Ok(Font::take(font))
        }
    }

    #[deprecated(note = "Use `font_family_by_name` instead.")]
    pub fn get_font_family_by_name(&self, family_name: &str) -> Option<FontFamily> {
        self.font_family_by_name(family_name).unwrap()
    }

    /// Find a [`FontFamily`] with the given name. Returns `None` if no family
    /// with that name is found.
    pub fn font_family_by_name(&self, family_name: &str) -> Result<Option<FontFamily>, HRESULT> {
        let mut index: u32 = 0;
        let mut exists: BOOL = FALSE;
        unsafe {
            self.native.FindFamilyName(PCWSTR(
                family_name.to_wide_null().as_ptr(),
            ), &mut index, &mut exists).map_err(|e| e.code())?;
            if exists == FALSE {
                return Ok(None);
            }

            let family = self.native.GetFontFamily(index).map_err(|e| e.code())?;
            Ok(Some(FontFamily::take(family)))
        }
    }
}
