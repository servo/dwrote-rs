/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::mem::MaybeUninit;
use std::os::raw::c_void;
use std::path::Path;
use std::path::PathBuf;
use std::ptr;
use std::slice;
use std::sync::Arc;

use windows::Win32::Foundation::FALSE;
use windows_core::{HRESULT, PCWSTR, Interface};
use windows::Win32::Graphics::DirectWrite::{IDWriteFontFile, DWRITE_FONT_FACE_TYPE, DWRITE_FONT_FACE_TYPE_UNKNOWN, DWRITE_FONT_FILE_TYPE_UNKNOWN, DWRITE_FONT_SIMULATIONS, IDWriteLocalFontFileLoader};

use super::{FontFace, DWriteFactory};
use crate::font_file_loader_impl::DataFontHelper;
use crate::helpers::ToWide;

#[derive(Clone)]
pub struct FontFile {
    native: IDWriteFontFile,
    // stream: Option<IDWriteFontFileStream>,
    data_key: usize,
    face_type: DWRITE_FONT_FACE_TYPE,
}

impl FontFile {
    pub fn new_from_path<P>(path: P) -> Option<FontFile>
    where
        P: AsRef<Path>,
    {
        unsafe {
            let font_file = DWriteFactory().CreateFontFileReference(
                PCWSTR(path.as_ref().as_os_str().to_wide_null().as_ptr()),
                None,
            ).ok()?;

            let mut ff = FontFile {
                native: font_file,
                // stream: None,
                data_key: 0,
                face_type: DWRITE_FONT_FACE_TYPE_UNKNOWN,
            };

            if ff.analyze() == 0 {
                None
            } else {
                Some(ff)
            }
        }
    }

    pub fn new_from_buffer(data: Arc<dyn AsRef<[u8]> + Sync + Send>) -> Option<FontFile> {
        let (font_file, _font_file_stream, key) = DataFontHelper::register_font_buffer(data);

        let mut ff = FontFile {
            native: font_file,
            // stream: Some(font_file_stream),
            data_key: key,
            face_type: DWRITE_FONT_FACE_TYPE_UNKNOWN,
        };

        if ff.analyze() == 0 {
            None
        } else {
            Some(ff)
        }
    }

    pub fn analyze_buffer(buffer: Arc<dyn AsRef<[u8]> + Sync + Send>) -> u32 {
        let (font_file, _font_file_stream, key) = DataFontHelper::register_font_buffer(buffer);

        let mut ff = FontFile {
            native: font_file,
            // stream: Some(font_file_stream),
            data_key: key,
            face_type: DWRITE_FONT_FACE_TYPE_UNKNOWN,
        };

        ff.analyze()
    }

    fn analyze(&mut self) -> u32 {
        let mut face_type = DWRITE_FONT_FACE_TYPE_UNKNOWN;
        let mut num_faces = 0;
        unsafe {
            let mut supported = FALSE;
            let mut _file_type = DWRITE_FONT_FILE_TYPE_UNKNOWN;

            let result = self.native.Analyze(
                &mut supported as *mut _,
                &mut _file_type,
                Some(&mut face_type),
                &mut num_faces,
            );
            if let Err(_) = result {
                return 0;
            }
        }
        self.face_type = face_type;
        num_faces
    }

    pub fn take(native: IDWriteFontFile) -> FontFile {
        let mut ff = FontFile {
            native,
            // stream: None,
            data_key: 0,
            face_type: DWRITE_FONT_FACE_TYPE_UNKNOWN,
        };
        ff.analyze();
        ff
    }

    pub fn data_key(&self) -> Option<usize> {
        if self.data_key != 0 {
            Some(self.data_key)
        } else {
            None
        }
    }

    pub(crate) fn as_ptr(&self) -> &IDWriteFontFile {
        &self.native
    }

    #[deprecated(note = "Use `font_file_bytes` instead.")]
    pub fn get_font_file_bytes(&self) -> Vec<u8> {
        self.font_file_bytes().unwrap()
    }

    // This is a helper to read the contents of this FontFile,
    // without requiring callers to deal with loaders, keys,
    // or streams.
    pub fn font_file_bytes(&self) -> Result<Vec<u8>, HRESULT> {
        unsafe {
            let mut ref_key: *const c_void = ptr::null();
            let mut ref_key_size: u32 = 0;
            self.native.GetReferenceKey(&mut ref_key as *mut _ as *mut *mut _, &mut ref_key_size).map_err(|e| e.code())?;
            let loader = self.native.GetLoader().map_err(|e| e.code())?;
            let stream = loader.CreateStreamFromKey(ref_key, ref_key_size).map_err(|e| e.code())?;

            let file_size = stream.GetFileSize().map_err(|e| e.code())?;
            
            let mut fragment_start = MaybeUninit::uninit();
            let mut fragment_context = MaybeUninit::uninit();
            stream.ReadFileFragment(fragment_start.as_mut_ptr(), 0, file_size, fragment_context.as_mut_ptr()).map_err(|e| e.code())?;

            let in_ptr = slice::from_raw_parts(fragment_start.assume_init() as *const u8, file_size as usize);
            let bytes = in_ptr.to_vec();

            stream.ReleaseFileFragment(fragment_context.assume_init());
            Ok(bytes)
        }
    }

    #[deprecated(note = "Use `font_file_path` instead.")]
    pub fn get_font_file_path(&self) -> Option<PathBuf> {
        self.font_file_path().ok()
    }

    // This is a helper to get the path of a font file,
    // without requiring callers to deal with loaders.
    pub fn font_file_path(&self) -> Result<PathBuf, HRESULT> {
        unsafe {
            let mut ref_key: *const c_void = ptr::null();
            let mut ref_key_size: u32 = 0;
            self.native.GetReferenceKey(&mut ref_key as *mut _ as *mut *mut _, &mut ref_key_size).map_err(|e| e.code())?;
            let loader = self.native.GetLoader().map_err(|e| e.code())?;
            let local_loader = loader.cast::<IDWriteLocalFontFileLoader>().map_err(|e| e.code())?;
            let len = local_loader.GetFilePathLengthFromKey(ref_key, ref_key_size).map_err(|e| e.code())?;
            let mut file_path = Vec::<u16>::with_capacity((len + 1) as usize);
            file_path.set_len((len + 1) as usize);
            local_loader.GetFilePathFromKey(ref_key, ref_key_size, &mut file_path).map_err(|e| e.code())?;
            Ok(PathBuf::from(String::from_utf16(&file_path).ok().unwrap()))
        }
    }

    pub fn create_face(
        &self,
        face_index: u32,
        simulations: DWRITE_FONT_SIMULATIONS,
    ) -> windows::core::Result<FontFace> {
        unsafe {
            let face = DWriteFactory()
                .CreateFontFace(self.face_type, &[None], face_index, simulations)
                .map_err(|e| e.code())?;
            Ok(FontFace::take(face))
        }
    }
}
