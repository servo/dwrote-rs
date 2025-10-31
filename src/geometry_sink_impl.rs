#![allow(non_snake_case, non_upper_case_globals)]

use std::sync::atomic::AtomicUsize;
use std::cell::RefCell;

use windows::Win32::Graphics::Direct2D::Common::{D2D1_BEZIER_SEGMENT, D2D1_FIGURE_BEGIN, D2D1_FIGURE_END, D2D1_FIGURE_END_CLOSED, D2D1_FILL_MODE, D2D1_PATH_SEGMENT, ID2D1SimplifiedGeometrySink, ID2D1SimplifiedGeometrySink_Impl};
use windows_core::implement;
use windows_numerics::Vector2;

use crate::outline_builder::OutlineBuilder;

// static GEOMETRY_SINK_VTBL: ID2D1SimplifiedGeometrySink_Vtbl = ID2D1SimplifiedGeometrySink_Vtbl {
//     parent: implement_iunknown!(static ID2D1SimplifiedGeometrySink, GeometrySinkImpl),
//     BeginFigure: GeometrySinkImpl_BeginFigure,
//     EndFigure: GeometrySinkImpl_EndFigure,
//     AddLines: GeometrySinkImpl_AddLines,
//     AddBeziers: GeometrySinkImpl_AddBeziers,
//     Close: GeometrySinkImpl_Close,
//     SetFillMode: GeometrySinkImpl_SetFillMode,
//     SetSegmentFlags: GeometrySinkImpl_SetSegmentFlags,
// };

#[implement(ID2D1SimplifiedGeometrySink)]
pub struct GeometrySinkImpl {
    // NB: This must be the first field.
    _refcount: AtomicUsize,
    outline_builder: RefCell<Box<dyn OutlineBuilder>>,
}

impl ID2D1SimplifiedGeometrySink_Impl for GeometrySinkImpl_Impl {
    fn SetFillMode(&self, _: D2D1_FILL_MODE) {
        // noop
    }

    fn SetSegmentFlags(&self, _: D2D1_PATH_SEGMENT) {
        // noop
    }
    fn BeginFigure(&self, start_point: &Vector2, _figurebegin: D2D1_FIGURE_BEGIN) {
        self.outline_builder.borrow_mut().move_to(start_point.X, start_point.Y);
    }
    fn AddLines(&self, points: *const Vector2, points_count: u32) {
        let points = unsafe { std::slice::from_raw_parts(points, points_count as usize) };
        let mut builder = self.outline_builder.borrow_mut();
        for point in points {
            builder.line_to(point.X, point.Y);
        }
    }
    fn AddBeziers(&self, beziers: *const D2D1_BEZIER_SEGMENT, beziers_count: u32) {
        let beziers = unsafe { std::slice::from_raw_parts(beziers, beziers_count as usize) };
        let mut builder = self.outline_builder.borrow_mut();
        for bezier in beziers {
            builder.curve_to(
                bezier.point1.X,
                bezier.point1.Y,
                bezier.point2.X,
                bezier.point2.Y,
                bezier.point3.X,
                bezier.point3.Y,
            );
        }
    }
    fn EndFigure(&self, figure_end: D2D1_FIGURE_END) {
        if figure_end == D2D1_FIGURE_END_CLOSED {
            self.outline_builder.borrow_mut().close();
        }
    }

    fn Close(&self) -> windows_core::Result<()> {
        Ok(())
    }
}

impl GeometrySinkImpl {
    pub fn new(outline_builder: Box<dyn OutlineBuilder>) -> GeometrySinkImpl {
        GeometrySinkImpl {
            _refcount: AtomicUsize::new(1),
            outline_builder: RefCell::new(outline_builder),
        }
    }
}
