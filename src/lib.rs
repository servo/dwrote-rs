/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(non_upper_case_globals)]

#[cfg(feature = "serde_serialization")]
extern crate serde;
#[cfg_attr(feature = "serde_serialization", macro_use)]
#[cfg(feature = "serde_serialization")]
extern crate serde_derive;

#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate windows;

include!("types.rs");

mod helpers;

#[cfg(test)]
mod test;

// We still use the DWrite structs for things like metrics; re-export them
// here
pub use windows::Win32::Foundation::RECT;
pub use windows::Win32::Graphics::DirectWrite::{
    DWRITE_TEXTURE_ALIASED_1x1, DWRITE_TEXTURE_CLEARTYPE_3x1, DWRITE_FONT_AXIS_VALUE,
    DWRITE_FONT_METRICS as FontMetrics0, DWRITE_FONT_METRICS1 as FontMetrics1,
    DWRITE_FONT_SIMULATIONS, DWRITE_FONT_SIMULATIONS_BOLD, DWRITE_FONT_SIMULATIONS_NONE,
    DWRITE_FONT_SIMULATIONS_OBLIQUE, DWRITE_GLYPH_OFFSET as GlyphOffset, DWRITE_GLYPH_RUN,
    DWRITE_MATRIX, DWRITE_MEASURING_MODE, DWRITE_MEASURING_MODE_GDI_CLASSIC,
    DWRITE_MEASURING_MODE_GDI_NATURAL, DWRITE_MEASURING_MODE_NATURAL, DWRITE_RENDERING_MODE,
    DWRITE_RENDERING_MODE_ALIASED, DWRITE_RENDERING_MODE_CLEARTYPE_GDI_CLASSIC,
    DWRITE_RENDERING_MODE_CLEARTYPE_GDI_NATURAL, DWRITE_RENDERING_MODE_CLEARTYPE_NATURAL,
    DWRITE_RENDERING_MODE_CLEARTYPE_NATURAL_SYMMETRIC, DWRITE_RENDERING_MODE_DEFAULT,
    DWRITE_RENDERING_MODE_GDI_CLASSIC, DWRITE_RENDERING_MODE_GDI_NATURAL,
    DWRITE_RENDERING_MODE_NATURAL, DWRITE_RENDERING_MODE_NATURAL_SYMMETRIC,
    DWRITE_RENDERING_MODE_OUTLINE, DWRITE_TEXTURE_TYPE,
};

mod bitmap_render_target;
pub use bitmap_render_target::BitmapRenderTarget;
mod font;
pub use font::{Font, FontMetrics, InformationalStringId};
mod font_collection;
pub use font_collection::FontCollection;
mod font_face;
pub use font_face::{FontFace, FontFaceType};
mod font_fallback;
pub use font_fallback::{FallbackResult, FontFallback};
mod font_family;
pub use font_family::FontFamily;
mod font_file;
pub use font_file::FontFile;
mod gdi_interop;
pub use gdi_interop::GdiInterop;
mod outline_builder;
pub use outline_builder::OutlineBuilder;
mod rendering_params;
pub use rendering_params::RenderingParams;
mod text_analysis_source;
pub use text_analysis_source::TextAnalysisSource;
mod glyph_run_analysis;
pub use glyph_run_analysis::GlyphRunAnalysis;

// This is an internal implementation of FontFileLoader, for our utility
// functions.  We don't wrap the DWriteFontFileLoader interface and
// related things.
mod font_file_loader_impl;

// This is an implementation of `FontCollectionLoader` for client code.
mod font_collection_impl;
pub use font_collection_impl::CustomFontCollectionLoaderImpl;

// This is an implementation of `TextAnalysisSource` for client code.
mod text_analysis_source_impl;
pub use text_analysis_source_impl::{
    CustomTextAnalysisSourceImpl, NumberSubstitution, TextAnalysisSourceMethods,
};

// This is an internal implementation of `GeometrySink` so that we can
// expose `IDWriteGeometrySink` in an idiomatic way.
// mod geometry_sink_impl;
