#![allow(non_snake_case, non_upper_case_globals)]

use std::collections::HashMap;
use std::marker::Send;
use std::mem;
use std::sync::atomic::AtomicUsize;
use std::sync::{atomic, Arc, Mutex};
use windows::core::Interface;
use windows::Win32::Foundation::E_INVALIDARG;
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory, IDWriteFontFile, IDWriteFontFileLoader, IDWriteFontFileLoader_Impl,
    IDWriteFontFileStream, IDWriteFontFileStream_Impl, DWRITE_FACTORY_TYPE_SHARED,
};

#[windows::core::implement(IDWriteFontFileLoader)]
struct FontFileLoader;

impl IDWriteFontFileLoader_Impl for FontFileLoader_Impl {
    fn CreateStreamFromKey(
        &self,
        fontfilereferencekey: *const core::ffi::c_void,
        fontfilereferencekeysize: u32,
    ) -> windows_core::Result<IDWriteFontFileStream> {
        if fontfilereferencekey.is_null() {
            return Err(E_INVALIDARG.into());
        }
        assert!(fontfilereferencekeysize == mem::size_of::<usize>() as u32);

        let key = unsafe { *(fontfilereferencekey as *const usize) };
        match FONT_FILE_STREAM_MAP.lock().unwrap().get(&key) {
            None => Err(E_INVALIDARG.into()),
            Some(&FontFileStreamPtr(ref file_stream)) => Ok(file_stream.clone()),
        }
    }
}

#[windows::core::implement(IDWriteFontFileStream)]
struct FontFileStream {
    refcount: atomic::AtomicUsize,
    key: usize,
    data: Arc<dyn AsRef<[u8]> + Sync + Send>,
}

impl FontFileStream {
    pub fn new(key: usize, data: Arc<dyn AsRef<[u8]> + Sync + Send>) -> FontFileStream {
        FontFileStream {
            refcount: AtomicUsize::new(1),
            key,
            data,
        }
    }
}

impl Drop for FontFileStream {
    fn drop(&mut self) {
        DataFontHelper::unregister_font_data(self.key);
    }
}

impl IDWriteFontFileStream_Impl for FontFileStream_Impl {
    fn ReadFileFragment(&self,fragmentstart: *mut *mut core::ffi::c_void,fileoffset:u64,fragmentsize:u64,fragmentcontext: *mut *mut core::ffi::c_void) -> windows_core::Result<()> {
        todo!()
    }

    fn ReleaseFileFragment(&self,fragmentcontext: *mut core::ffi::c_void) {
        todo!()
    }

    fn GetFileSize(&self) -> windows_core::Result<u64> {
        todo!()
    }

    fn GetLastWriteTime(&self) -> windows_core::Result<u64> {
        todo!()
    }
}

struct FontFileStreamPtr(IDWriteFontFileStream);

unsafe impl Send for FontFileStreamPtr {}

static mut FONT_FILE_KEY: atomic::AtomicUsize = AtomicUsize::new(0);

#[derive(Clone)]
struct FontFileLoaderWrapper(IDWriteFontFileLoader);

unsafe impl Send for FontFileLoaderWrapper {}
unsafe impl Sync for FontFileLoaderWrapper {}

lazy_static! {
    static ref FONT_FILE_STREAM_MAP: Mutex<HashMap<usize, FontFileStreamPtr>> =
        Mutex::new(HashMap::new());
    static ref FONT_FILE_LOADER: Mutex<FontFileLoaderWrapper> = {
        unsafe {
            let factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).unwrap();
            let ffl = FontFileLoader {};
            let ffl: IDWriteFontFileLoader = ffl.cast().unwrap();
            factory.RegisterFontFileLoader(&ffl).unwrap();
            Mutex::new(FontFileLoaderWrapper(ffl))
        }
    };
}

pub(crate) struct DataFontHelper;

impl DataFontHelper {
    pub(crate) fn register_font_buffer(
        font_data: Arc<dyn AsRef<[u8]> + Sync + Send>,
    ) -> (IDWriteFontFile, IDWriteFontFileStream, usize) {
        unsafe {
            let key = FONT_FILE_KEY.fetch_add(1, atomic::Ordering::Relaxed);
            let font_file_stream = FontFileStream::new(key, font_data);
            let font_file_stream: IDWriteFontFileStream = font_file_stream.cast().unwrap();

            {
                let mut map = FONT_FILE_STREAM_MAP.lock().unwrap();
                map.insert(key, FontFileStreamPtr(font_file_stream.clone()));
            }

            let factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).unwrap();
            let loader = FONT_FILE_LOADER.lock().unwrap();
            let font_file = factory
                .CreateCustomFontFileReference(
                    mem::transmute(&key),
                    mem::size_of::<usize>() as u32,
                    &loader.0,
                )
                .unwrap();

            (font_file, font_file_stream, key)
        }
    }

    fn unregister_font_data(key: usize) {
        let mut map = FONT_FILE_STREAM_MAP.lock().unwrap();
        if map.remove(&key).is_none() {
            panic!("unregister_font_data: trying to unregister key that is no longer registered");
        }
    }
}
