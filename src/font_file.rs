/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::ffi::{c_void, OsString};
use std::os::windows::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::ptr;
use std::slice;
use std::sync::Arc;
use windows::core::{Interface, HSTRING};
use windows::Win32::Foundation::FALSE;
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory, IDWriteFontFile, IDWriteFontFileStream,
    IDWriteLocalFontFileLoader, DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_FACE_TYPE,
    DWRITE_FONT_FACE_TYPE_UNKNOWN, DWRITE_FONT_FILE_TYPE_UNKNOWN, DWRITE_FONT_SIMULATIONS,
};

use crate::font_face::FontFace;
use crate::font_file_loader_impl::DataFontHelper;

pub struct FontFile {
    pub(crate) native: IDWriteFontFile,
    stream: Option<IDWriteFontFileStream>,
    data_key: usize,
    face_type: DWRITE_FONT_FACE_TYPE,
}

impl FontFile {
    pub fn new_from_path<P>(path: P) -> Option<FontFile>
    where
        P: AsRef<Path>,
    {
        unsafe {
            let factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).ok()?;
            let font_file = factory
                .CreateFontFileReference(&HSTRING::from(path.as_ref()), None)
                .ok()?;

            let mut ff = FontFile {
                native: font_file,
                stream: None,
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
        let (font_file, font_file_stream, key) = DataFontHelper::register_font_buffer(data);

        let mut ff = FontFile {
            native: font_file,
            stream: Some(font_file_stream),
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
        let (font_file, font_file_stream, key) = DataFontHelper::register_font_buffer(buffer);

        let mut ff = FontFile {
            native: font_file,
            stream: Some(font_file_stream),
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

            if let Ok(_) = self.native.Analyze(
                &mut supported,
                &mut _file_type,
                Some(&mut face_type),
                &mut num_faces,
            ) {
                if supported == FALSE {
                    return 0;
                }
            } else {
                return 0;
            }
        }
        self.face_type = face_type;
        num_faces
    }

    pub fn take(native: IDWriteFontFile) -> FontFile {
        let mut ff = FontFile {
            native,
            stream: None,
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

    // This is a helper to read the contents of this FontFile,
    // without requiring callers to deal with loaders, keys,
    // or streams.
    pub fn get_font_file_bytes(&self) -> Vec<u8> {
        unsafe {
            let mut ref_key: *mut c_void = ptr::null_mut();
            let mut ref_key_size: u32 = 0;
            self.native
                .GetReferenceKey(&mut ref_key, &mut ref_key_size)
                .unwrap();

            let loader = self.native.GetLoader().unwrap();

            let stream = loader.CreateStreamFromKey(ref_key, ref_key_size).unwrap();

            let file_size = stream.GetFileSize().unwrap();

            let mut fragment_start: *mut c_void = ptr::null_mut();
            let mut fragment_context: *mut c_void = ptr::null_mut();
            stream
                .ReadFileFragment(&mut fragment_start, 0, file_size, &mut fragment_context)
                .unwrap();

            let in_ptr = slice::from_raw_parts(fragment_start as *const u8, file_size as usize);
            let bytes = in_ptr.to_vec();

            stream.ReleaseFileFragment(fragment_context);

            bytes
        }
    }

    // This is a helper to get the path of a font file,
    // without requiring callers to deal with loaders.
    pub fn get_font_file_path(&self) -> Option<PathBuf> {
        unsafe {
            let mut ref_key: *mut c_void = ptr::null_mut();
            let mut ref_key_size: u32 = 0;
            self.native
                .GetReferenceKey(&mut ref_key, &mut ref_key_size)
                .ok()?;

            let loader = self.native.GetLoader().ok()?;

            let local_loader = match loader.cast::<IDWriteLocalFontFileLoader>() {
                Ok(local_loader) => local_loader,
                Err(_) => return None,
            };

            let file_path_len = local_loader
                .GetFilePathLengthFromKey(ref_key, ref_key_size)
                .ok()?;

            let mut file_path_buf = vec![0; file_path_len as usize + 1];
            local_loader
                .GetFilePathFromKey(ref_key, ref_key_size, &mut file_path_buf)
                .ok()?;

            if let Some(&0) = file_path_buf.last() {
                file_path_buf.pop();
            }

            Some(PathBuf::from(OsString::from_wide(&file_path_buf)))
        }
    }

    pub fn create_face(
        &self,
        face_index: u32,
        simulations: DWRITE_FONT_SIMULATIONS,
    ) -> windows::core::Result<FontFace> {
        unsafe {
            let factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;
            let face = factory.CreateFontFace(
                self.face_type,
                &[Some(self.native.clone())],
                face_index,
                simulations,
            )?;
            Ok(FontFace::take(face))
        }
    }
}

impl Clone for FontFile {
    fn clone(&self) -> FontFile {
        FontFile {
            native: self.native.clone(),
            stream: self.stream.clone(),
            data_key: self.data_key,
            face_type: self.face_type,
        }
    }
}
