/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::ffi::c_void;
use std::mem::{MaybeUninit, zeroed};
use std::slice;
use std::{error, fmt, ptr};
use windows::Win32::Foundation::{FALSE, TRUE};

use windows::Win32::Graphics::Direct2D::Common::ID2D1SimplifiedGeometrySink;
use windows::Win32::Graphics::DirectWrite::{
    DWRITE_FONT_AXIS_ATTRIBUTES_NONE, DWRITE_FONT_AXIS_ATTRIBUTES_VARIABLE, DWRITE_FONT_AXIS_TAG, DWRITE_FONT_AXIS_VALUE, DWRITE_FONT_FACE_TYPE_BITMAP, DWRITE_FONT_FACE_TYPE_CFF, DWRITE_FONT_FACE_TYPE_RAW_CFF, DWRITE_FONT_FACE_TYPE_TRUETYPE, DWRITE_FONT_FACE_TYPE_TRUETYPE_COLLECTION, DWRITE_FONT_FACE_TYPE_TYPE1, DWRITE_FONT_FACE_TYPE_VECTOR, DWRITE_FONT_SIMULATIONS, DWRITE_GLYPH_METRICS, DWRITE_GLYPH_OFFSET, DWRITE_MATRIX, DWRITE_MEASURING_MODE, DWRITE_RENDERING_MODE, DWRITE_RENDERING_MODE_DEFAULT, IDWriteFontFace, IDWriteFontFace1, IDWriteFontFace5, IDWriteFontFile, IDWriteRenderingParams};
use windows_core::{HRESULT, Interface};
use super::{DWriteFactory, FontFile, FontMetrics, DefaultDWriteRenderParams, FontSimulations};
use crate::geometry_sink_impl::GeometrySinkImpl;
use crate::outline_builder::OutlineBuilder;

#[derive(Clone)]
pub struct FontFace {
    native: IDWriteFontFace,
    face1: Option<IDWriteFontFace1>,
    face5: Option<IDWriteFontFace5>,
}

impl FontFace {
    pub fn take(native: IDWriteFontFace) -> FontFace {
        FontFace {
            native,
            face1: None,
            face5: None,
        }
    }

    pub unsafe fn as_ptr(&self) -> &IDWriteFontFace {
        &self.native
    }

    unsafe fn raw_files(&self) -> Result<Vec<Option<IDWriteFontFile>>, HRESULT> {
        unsafe {
            let mut number_of_files: u32 = 0;
            self.native.GetFiles(&mut number_of_files, None).map_err(|e| e.code())?;
            let mut file_ptrs: Vec<Option<IDWriteFontFile>> = Vec::with_capacity(number_of_files as usize);
            file_ptrs.set_len(number_of_files as usize);
            self.native.GetFiles(&mut number_of_files, Some(file_ptrs.as_mut_ptr())).map_err(|e| e.code())?;
            Ok(file_ptrs)
        }
    }

    #[deprecated(note = "Use `files` instead.")]
    pub fn get_files(&self) -> Vec<FontFile> {
        self.files().unwrap()
    }

    pub fn files(&self) -> Result<Vec<FontFile>, HRESULT> {
        unsafe {
            self.raw_files().map(|file_ptrs| {
                file_ptrs
                    .into_iter()
                    .flatten()
                    .map(|f| FontFile::take(f))
                    .collect::<Vec<FontFile>>()
            })
        }
    }

    pub fn create_font_face_with_simulations(
        &self,
        simulations: DWRITE_FONT_SIMULATIONS,
    ) -> FontFace {
        unsafe {
            let files = self.raw_files().unwrap();
            let face_type = self.native.GetType();
            let face_index = self.native.GetIndex();
            let face = DWriteFactory().CreateFontFace(face_type, &files, face_index, simulations).unwrap();
            FontFace::take(face)
        }
    }

    pub fn get_glyph_count(&self) -> u16 {
        unsafe { self.native.GetGlyphCount() }
    }

    pub fn metrics(&self) -> FontMetrics {
        unsafe {
            let font_1 = &self.face1;
            match font_1 {
                None => {
                    let mut metrics = MaybeUninit::uninit();
                    self.native.GetMetrics(metrics.as_mut_ptr());
                    FontMetrics::Metrics0(metrics.assume_init())
                }
                Some(font_1) => {
                    let mut metrics_1 = MaybeUninit::uninit();
                    font_1.GetMetrics(metrics_1.as_mut_ptr());
                    FontMetrics::Metrics1(metrics_1.assume_init())
                }
            }
        }
    }

    #[deprecated(note = "Use `glyph_indices` instead.")]
    pub fn get_glyph_indices(&self, code_points: &[u32]) -> Vec<u16> {
        self.glyph_indices(code_points).unwrap()
    }

    pub fn glyph_indices(&self, code_points: &[u32]) -> Result<Vec<u16>, HRESULT> {
        unsafe {
            let mut glyph_indices: Vec<u16> = vec![0; code_points.len()];
            self.native.GetGlyphIndices(
                code_points.as_ptr(),
                code_points.len() as u32,
                glyph_indices.as_mut_ptr(),
            ).map_err(|e| e.code())?;
            Ok(glyph_indices)
        }
    }

    #[deprecated(note = "Use `design_glyph_metrics` instead.")]
    pub fn get_design_glyph_metrics(
        &self,
        glyph_indices: &[u16],
        is_sideways: bool,
    ) -> Vec<DWRITE_GLYPH_METRICS> {
        self.design_glyph_metrics(glyph_indices, is_sideways)
            .unwrap()
    }

    pub fn design_glyph_metrics(
        &self,
        glyph_indices: &[u16],
        is_sideways: bool,
    ) -> Result<Vec<DWRITE_GLYPH_METRICS>, HRESULT> {
        unsafe {
            let mut metrics: Vec<DWRITE_GLYPH_METRICS> = vec![zeroed(); glyph_indices.len()];
            self.native.GetDesignGlyphMetrics(
                glyph_indices.as_ptr(),
                glyph_indices.len() as u32,
                metrics.as_mut_ptr(),
                is_sideways,
            ).map_err(|e| e.code())?;
            Ok(metrics)
        }
    }

    #[deprecated(note = "Use `gdi_compatible_glyph_metrics` instead.")]
    pub fn get_gdi_compatible_glyph_metrics(
        &self,
        em_size: f32,
        pixels_per_dip: f32,
        transform: *const DWRITE_MATRIX,
        use_gdi_natural: bool,
        glyph_indices: &[u16],
        is_sideways: bool,
    ) -> Vec<DWRITE_GLYPH_METRICS> {
        self.gdi_compatible_glyph_metrics(
            em_size,
            pixels_per_dip,
            transform,
            use_gdi_natural,
            glyph_indices,
            is_sideways,
        )
        .unwrap()
    }

    pub fn gdi_compatible_glyph_metrics(
        &self,
        em_size: f32,
        pixels_per_dip: f32,
        transform: *const DWRITE_MATRIX,
        use_gdi_natural: bool,
        glyph_indices: &[u16],
        is_sideways: bool,
    ) -> Result<Vec<DWRITE_GLYPH_METRICS>, HRESULT> {
        unsafe {
            let mut metrics: Vec<DWRITE_GLYPH_METRICS> = vec![zeroed(); glyph_indices.len()];
            self.native.GetGdiCompatibleGlyphMetrics(
                em_size,
                pixels_per_dip,
                Some(transform),
                use_gdi_natural,
                glyph_indices.as_ptr(),
                glyph_indices.len() as u32,
                metrics.as_mut_ptr(),
                is_sideways,
            ).map_err(|e| e.code())?;
            Ok(metrics)
        }
    }

    #[deprecated(note = "Use `font_table` instead.")]
    pub fn get_font_table(&self, opentype_table_tag: u32) -> Option<Vec<u8>> {
        self.font_table(opentype_table_tag).unwrap()
    }

    /// Returns the contents of the OpenType table with the given tag.
    ///
    /// NB: The bytes of the tag are reversed! You probably want to use the `u32::swap_bytes()`
    /// method on the tag value before calling this method.
    pub fn font_table(&self, opentype_table_tag: u32) -> Result<Option<Vec<u8>>, HRESULT> {
        unsafe {
            let mut table_data_ptr: *const u8 = ptr::null_mut();
            let mut table_size: u32 = 0;
            let mut table_context: *mut c_void = ptr::null_mut();
            let mut exists = FALSE;
            self.native.TryGetFontTable(
                opentype_table_tag,
                &mut table_data_ptr as *mut *const _ as *mut *mut c_void,
                &mut table_size,
                &mut table_context,
                &mut exists,
            ).map_err(|e| e.code())?;
            if exists == FALSE {
                return Ok(None);
            }
            let table_bytes = slice::from_raw_parts(table_data_ptr, table_size as usize).to_vec();
            self.native.ReleaseFontTable(table_context);
            Ok(Some(table_bytes))
        }
    }

    pub fn get_recommended_rendering_mode(
        &self,
        em_size: f32,
        pixels_per_dip: f32,
        measure_mode: DWRITE_MEASURING_MODE,
        rendering_params: &IDWriteRenderingParams,
    ) -> DWRITE_RENDERING_MODE {
        unsafe {
            self.native.GetRecommendedRenderingMode(
                em_size,
                pixels_per_dip,
                measure_mode,
                rendering_params,
            ).unwrap_or(DWRITE_RENDERING_MODE_DEFAULT)
        }
    }

    pub fn get_recommended_rendering_mode_default_params(
        &self,
        em_size: f32,
        pixels_per_dip: f32,
        measure_mode: DWRITE_MEASURING_MODE,
    ) -> DWRITE_RENDERING_MODE {
        self.get_recommended_rendering_mode(
            em_size,
            pixels_per_dip,
            measure_mode,
            DefaultDWriteRenderParams()
        )
    }

    #[deprecated(note = "Use `glyph_run_outline` instead.")]
    pub fn get_glyph_run_outline(
        &self,
        em_size: f32,
        glyph_indices: &[u16],
        glyph_advances: Option<&[f32]>,
        glyph_offsets: Option<&[DWRITE_GLYPH_OFFSET]>,
        is_sideways: bool,
        is_right_to_left: bool,
        outline_builder: Box<dyn OutlineBuilder>,
    ) {
        self.glyph_run_outline(
            em_size,
            glyph_indices,
            glyph_advances,
            glyph_offsets,
            is_sideways,
            is_right_to_left,
            outline_builder,
        )
        .unwrap()
    }

    pub fn glyph_run_outline(
        &self,
        em_size: f32,
        glyph_indices: &[u16],
        glyph_advances: Option<&[f32]>,
        glyph_offsets: Option<&[DWRITE_GLYPH_OFFSET]>,
        is_sideways: bool,
        is_right_to_left: bool,
        outline_builder: Box<dyn OutlineBuilder>,
    ) -> Result<(), GlyphRunOutlineError> {
        let glyph_advances = match glyph_advances {
            None => ptr::null(),
            Some(glyph_advances) => {
                if glyph_advances.len() != glyph_indices.len() {
                    return Err(GlyphRunOutlineError::InvalidInput);
                }
                glyph_advances.as_ptr()
            }
        };
        let glyph_offsets = match glyph_offsets {
            None => ptr::null(),
            Some(glyph_offsets) => {
                if glyph_offsets.len() != glyph_indices.len() {
                    return Err(GlyphRunOutlineError::InvalidInput);
                }
                glyph_offsets.as_ptr()
            }
        };
        let geometry_sink: ID2D1SimplifiedGeometrySink = GeometrySinkImpl::new(outline_builder).into();
        unsafe {
            self.native.GetGlyphRunOutline(
                em_size,
                glyph_indices.as_ptr(),
                Some(glyph_advances),
                Some(glyph_offsets),
                glyph_indices.len() as u32,
                is_sideways,
                is_right_to_left,
                &geometry_sink,
            ).map_err(|e| GlyphRunOutlineError::Win32Error(e.code()))?;
        }
        Ok(())
    }

    pub fn has_kerning_pairs(&self) -> bool {
        unsafe {
            match &self.face1 {
                Some(face1) => face1.HasKerningPairs() == TRUE,
                None => false,
            }
        }
    }

    #[deprecated(note = "Use `glyph_pair_kerning_adjustment` instead.")]
    pub fn get_glyph_pair_kerning_adjustment(&self, first_glyph: u16, second_glyph: u16) -> i32 {
        self.glyph_pair_kerning_adjustment(first_glyph, second_glyph)
            .unwrap()
    }

    pub fn glyph_pair_kerning_adjustment(
        &self,
        first_glyph: u16,
        second_glyph: u16,
    ) -> Result<i32, HRESULT> {
        unsafe {
            match &self.face1 {
                Some(face1) => {
                    let mut adjustments = [0; 2];
                    face1.GetKerningPairAdjustments(
                        2,
                        [first_glyph, second_glyph].as_ptr(),
                        adjustments.as_mut_ptr(),
                    ).map_err(|e| e.code())?;

                    Ok(adjustments[0])
                }
                None => Ok(0),
            }
        }
    }

    #[inline]
    pub fn get_type(&self) -> FontFaceType {
        unsafe {
            match self.native.GetType() {
                DWRITE_FONT_FACE_TYPE_CFF => FontFaceType::Cff,
                DWRITE_FONT_FACE_TYPE_RAW_CFF => FontFaceType::RawCff,
                DWRITE_FONT_FACE_TYPE_TRUETYPE => FontFaceType::TrueType,
                DWRITE_FONT_FACE_TYPE_TRUETYPE_COLLECTION => FontFaceType::TrueTypeCollection,
                DWRITE_FONT_FACE_TYPE_TYPE1 => FontFaceType::Type1,
                DWRITE_FONT_FACE_TYPE_VECTOR => FontFaceType::Vector,
                DWRITE_FONT_FACE_TYPE_BITMAP => FontFaceType::Bitmap,
                _ => FontFaceType::Unknown,
            }
        }
    }

    #[inline]
    pub fn get_index(&self) -> u32 {
        unsafe { self.native.GetIndex() }
    }

    pub fn has_variations(&self) -> bool {
        unsafe {
            if let Some(face5) = &self.face5 {
                face5.HasVariations() == TRUE
            } else {
                false
            }
        }
    }

    /// If this font has variations, return a [`Vec<DWRITE_FONT_AXIS_VALUE`] of the
    /// variation axes and their values. If the font does not have variations,
    /// return an empty `Vec`.
    pub fn variations(&self) -> Result<Vec<DWRITE_FONT_AXIS_VALUE>, HRESULT> {
        let Some(face5) = &self.face5 else {
            return Ok(vec![]);
        };
        if unsafe { face5.HasVariations() != TRUE } {
            return Ok(vec![]);
        }
        let axis_count = unsafe { face5.GetFontAxisValueCount() as usize };
        if axis_count == 0 {
            return Ok(vec![]);
        }

        let resource = unsafe { face5.GetFontResource().map_err(|e| e.code())? };

        let mut axis_values = Vec::with_capacity(axis_count);
        axis_values.resize(
            axis_count,
            DWRITE_FONT_AXIS_VALUE {
                axisTag: DWRITE_FONT_AXIS_TAG(0),
                value: 0.,
            },
        );

        unsafe { face5.GetFontAxisValues(&mut axis_values).map_err(|e| e.code())? };

        Ok(axis_values
            .iter()
            .enumerate()
            .filter_map(|(index, axis_value)| {
                let attributes = unsafe { resource.GetFontAxisAttributes(index as u32) };
                if attributes & DWRITE_FONT_AXIS_ATTRIBUTES_VARIABLE == DWRITE_FONT_AXIS_ATTRIBUTES_NONE {
                    None
                } else {
                    Some(*axis_value)
                }
            })
            .collect())
    }

    pub fn create_font_face_with_variations(
        &self,
        simulations: DWRITE_FONT_SIMULATIONS,
        axis_values: &[DWRITE_FONT_AXIS_VALUE],
    ) -> Option<FontFace> {
        unsafe {
            if let Some(face5) = &self.face5 {
                let resource = face5.GetFontResource().ok()?;
                let face = resource.CreateFontFace(simulations, axis_values).ok()?;
                let face = face.cast::<IDWriteFontFace>().unwrap();
                return Some(FontFace::take(face));
            }
            None
        }
    }

    pub fn simulations(&self) -> FontSimulations {
        unsafe { self.native.GetSimulations().into() }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FontFaceType {
    Unknown,
    Cff,
    RawCff,
    TrueType,
    TrueTypeCollection,
    Type1,
    Vector,
    Bitmap,
}

#[derive(Debug)]
pub enum GlyphRunOutlineError {
    InvalidInput,
    Win32Error(HRESULT),
}

impl fmt::Display for GlyphRunOutlineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput => write!(f, "Invalid input"),
            Self::Win32Error(code) => write!(f, "{:#x}", code.0),
        }
    }
}

impl error::Error for GlyphRunOutlineError {}
