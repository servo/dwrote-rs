/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::ffi::c_void;
use std::mem::{self, zeroed};
use std::ptr;
use std::slice;
use windows::core::Interface;
use windows::Win32::Foundation::{BOOL, FALSE, TRUE};
use windows::Win32::Graphics::DirectWrite::IDWriteRenderingParams;
use windows::Win32::Graphics::DirectWrite::DWRITE_FONT_FACE_TYPE_TRUETYPE;
use windows::Win32::Graphics::DirectWrite::DWRITE_RENDERING_MODE_NATURAL_SYMMETRIC;
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory, DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_SIMULATIONS_NONE,
    DWRITE_MEASURING_MODE,
};
use windows::Win32::Graphics::DirectWrite::{IDWriteFontFace, IDWriteFontFile};
use windows::Win32::Graphics::DirectWrite::{
    IDWriteFontFace1, IDWriteFontFace5, DWRITE_FONT_AXIS_VALUE,
};
use windows::Win32::Graphics::DirectWrite::{
    DWRITE_FONT_FACE_TYPE_BITMAP, DWRITE_FONT_FACE_TYPE_CFF,
};
use windows::Win32::Graphics::DirectWrite::{
    DWRITE_FONT_FACE_TYPE_RAW_CFF, DWRITE_FONT_FACE_TYPE_TYPE1,
};
use windows::Win32::Graphics::DirectWrite::{
    DWRITE_FONT_FACE_TYPE_TRUETYPE_COLLECTION, DWRITE_FONT_FACE_TYPE_VECTOR,
};
use windows::Win32::Graphics::DirectWrite::{DWRITE_FONT_SIMULATIONS, DWRITE_GLYPH_METRICS};
use windows::Win32::Graphics::DirectWrite::{
    DWRITE_GLYPH_OFFSET, DWRITE_MATRIX, DWRITE_RENDERING_MODE,
};

use super::{FontFile, FontMetrics};
// use crate::geometry_sink_impl::GeometrySinkImpl;
use crate::outline_builder::OutlineBuilder;

pub struct FontFace {
    pub(crate) native: IDWriteFontFace,
    face1: Option<IDWriteFontFace1>,
    face5: Option<IDWriteFontFace5>,
}

impl FontFace {
    pub fn take(native: IDWriteFontFace) -> FontFace {
        let cell = native;
        FontFace {
            native: cell,
            face1: None,
            face5: None,
        }
    }

    fn get_raw_files(&self) -> Vec<Option<IDWriteFontFile>> {
        unsafe {
            let mut number_of_files: u32 = 0;
            self.native.GetFiles(&mut number_of_files, None).unwrap();
            if number_of_files == 0 {
                return vec![];
            }

            let mut files = vec![None; number_of_files as usize];
            self.native
                .GetFiles(&mut number_of_files, Some(files.as_mut_ptr()))
                .unwrap();
            files
        }
    }

    pub fn get_files(&self) -> Vec<FontFile> {
        let files = self.get_raw_files();
        files
            .into_iter()
            .filter_map(|f| f.map(FontFile::take))
            .collect()
    }

    pub fn create_font_face_with_simulations(
        &self,
        simulations: DWRITE_FONT_SIMULATIONS,
    ) -> FontFace {
        unsafe {
            let factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).unwrap();
            let file_ptrs = self.get_raw_files();
            let face_type = self.native.GetType();
            let face_index = self.native.GetIndex();
            let face = factory
                .CreateFontFace(
                    face_type,
                    &file_ptrs,
                    face_index,
                    DWRITE_FONT_SIMULATIONS_NONE,
                )
                .unwrap();
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
                    let mut metrics = mem::zeroed();
                    self.native.GetMetrics(&mut metrics);
                    FontMetrics::Metrics0(metrics)
                }
                Some(font_1) => {
                    let mut metrics_1 = mem::zeroed();
                    font_1.GetMetrics(&mut metrics_1);
                    FontMetrics::Metrics1(metrics_1)
                }
            }
        }
    }

    pub fn get_glyph_indices(&self, code_points: &[u32]) -> Vec<u16> {
        unsafe {
            let mut glyph_indices: Vec<u16> = vec![0; code_points.len()];
            self.native
                .GetGlyphIndices(
                    code_points.as_ptr(),
                    code_points.len() as u32,
                    glyph_indices.as_mut_ptr(),
                )
                .unwrap();
            glyph_indices
        }
    }

    pub fn get_design_glyph_metrics(
        &self,
        glyph_indices: &[u16],
        is_sideways: bool,
    ) -> Vec<DWRITE_GLYPH_METRICS> {
        unsafe {
            let mut metrics: Vec<DWRITE_GLYPH_METRICS> = vec![zeroed(); glyph_indices.len()];
            self.native
                .GetDesignGlyphMetrics(
                    glyph_indices.as_ptr(),
                    glyph_indices.len() as u32,
                    metrics.as_mut_ptr(),
                    is_sideways,
                )
                .unwrap();
            metrics
        }
    }

    pub fn get_gdi_compatible_glyph_metrics(
        &self,
        em_size: f32,
        pixels_per_dip: f32,
        transform: *const DWRITE_MATRIX,
        use_gdi_natural: bool,
        glyph_indices: &[u16],
        is_sideways: bool,
    ) -> Vec<DWRITE_GLYPH_METRICS> {
        unsafe {
            let mut metrics: Vec<DWRITE_GLYPH_METRICS> = vec![zeroed(); glyph_indices.len()];
            self.native
                .GetGdiCompatibleGlyphMetrics(
                    em_size,
                    pixels_per_dip,
                    Some(transform),
                    use_gdi_natural,
                    glyph_indices.as_ptr(),
                    glyph_indices.len() as u32,
                    metrics.as_mut_ptr(),
                    is_sideways,
                )
                .unwrap();
            metrics
        }
    }

    /// Returns the contents of the OpenType table with the given tag.
    ///
    /// NB: The bytes of the tag are reversed! You probably want to use the `u32::swap_bytes()`
    /// method on the tag value before calling this method.
    pub fn get_font_table(&self, opentype_table_tag: u32) -> Option<Vec<u8>> {
        unsafe {
            let mut table_data_ptr: *const u8 = ptr::null_mut();
            let mut table_size: u32 = 0;
            let mut table_context: *mut c_void = ptr::null_mut();
            let mut exists: BOOL = FALSE;

            self.native
                .TryGetFontTable(
                    opentype_table_tag,
                    &mut table_data_ptr as *mut *const _ as *mut *mut c_void,
                    &mut table_size,
                    &mut table_context,
                    &mut exists,
                )
                .unwrap();

            if exists == FALSE {
                return None;
            }

            let table_bytes = slice::from_raw_parts(table_data_ptr, table_size as usize).to_vec();

            self.native.ReleaseFontTable(table_context);

            Some(table_bytes)
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
            self.native
                .GetRecommendedRenderingMode(
                    em_size,
                    pixels_per_dip,
                    measure_mode,
                    rendering_params,
                )
                .unwrap_or(DWRITE_RENDERING_MODE_NATURAL_SYMMETRIC)
        }
    }

    pub fn get_recommended_rendering_mode_default_params(
        &self,
        em_size: f32,
        pixels_per_dip: f32,
        measure_mode: DWRITE_MEASURING_MODE,
    ) -> DWRITE_RENDERING_MODE {
        unsafe {
            let factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).unwrap();
            self.get_recommended_rendering_mode(
                em_size,
                pixels_per_dip,
                measure_mode,
                &factory.CreateRenderingParams().unwrap(),
            )
        }
    }

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
        todo!();
        // unsafe {
        //     let glyph_advances = match glyph_advances {
        //         None => ptr::null(),
        //         Some(glyph_advances) => {
        //             assert_eq!(glyph_advances.len(), glyph_indices.len());
        //             glyph_advances.as_ptr()
        //         }
        //     };
        //     let glyph_offsets = match glyph_offsets {
        //         None => ptr::null(),
        //         Some(glyph_offsets) => {
        //             assert_eq!(glyph_offsets.len(), glyph_indices.len());
        //             glyph_offsets.as_ptr()
        //         }
        //     };
        //     let is_sideways = BOOL::from(is_sideways);
        //     let is_right_to_left = BOOL::from(is_right_to_left);
        //     let geometry_sink = GeometrySinkImpl::new(outline_builder);
        //     self.native
        //         .GetGlyphRunOutline(
        //             em_size,
        //             glyph_indices.as_ptr(),
        //             Some(glyph_advances),
        //             Some(glyph_offsets),
        //             glyph_indices.len() as u32,
        //             is_sideways,
        //             is_right_to_left,
        //             geometry_sink,
        //         )
        //         .unwrap();
        // }
    }

    pub fn has_kerning_pairs(&self) -> bool {
        unsafe {
            match &self.face1 {
                Some(face1) => face1.HasKerningPairs() == TRUE,
                None => false,
            }
        }
    }

    pub fn get_glyph_pair_kerning_adjustment(&self, first_glyph: u16, second_glyph: u16) -> i32 {
        unsafe {
            match &self.face1 {
                Some(face1) => {
                    let mut adjustments = [0; 2];
                    face1
                        .GetKerningPairAdjustments(
                            2,
                            [first_glyph, second_glyph].as_ptr(),
                            adjustments.as_mut_ptr(),
                        )
                        .unwrap();

                    adjustments[0]
                }
                None => 0,
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

    pub fn create_font_face_with_variations(
        &self,
        simulations: DWRITE_FONT_SIMULATIONS,
        axis_values: &[DWRITE_FONT_AXIS_VALUE],
    ) -> Option<FontFace> {
        unsafe {
            let face5 = &self.face5;
            let face5 = match face5 {
                None => return None,
                Some(face5) => face5,
            };
            let resource = face5.GetFontResource().ok()?;
            let var_face = resource.CreateFontFace(simulations, &axis_values).ok()?;
            let var_face = (var_face).cast().ok()?;
            Some(FontFace::take(var_face))
        }
    }
}

impl Clone for FontFace {
    fn clone(&self) -> FontFace {
        unsafe {
            FontFace {
                native: self.native.clone(),
                face1: self.face1.clone(),
                face5: self.face5.clone(),
            }
        }
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
