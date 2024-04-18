use std::{
    ffi::{c_int, c_void},
    mem,
};

use libtess2_sys::{
    tessAddContour, tessDeleteTess, tessGetElementCount, tessGetElements, tessGetVertexCount,
    tessGetVertices, tessNewTess, tessSetOption, tessTesselate, TESSreal,
    TESStesselator, TessElementType, TessElementType_TESS_BOUNDARY_CONTOURS,
    TessElementType_TESS_CONNECTED_POLYGONS, TessElementType_TESS_POLYGONS, TessOption,
    TessOption_TESS_CONSTRAINED_DELAUNAY_TRIANGULATION, TessOption_TESS_REVERSE_CONTOURS,
    TessWindingRule, TessWindingRule_TESS_WINDING_ABS_GEQ_TWO,
    TessWindingRule_TESS_WINDING_NEGATIVE, TessWindingRule_TESS_WINDING_NONZERO,
    TessWindingRule_TESS_WINDING_ODD, TessWindingRule_TESS_WINDING_POSITIVE,
};

#[derive(Clone, Copy)]
pub enum WindingRule {
    EvenOdd,
    NonZero,
    Positive,
    Negative,
    AbsGeqTwo,
}

impl From<WindingRule> for TessWindingRule {
    fn from(value: WindingRule) -> Self {
        match value {
            WindingRule::EvenOdd => TessWindingRule_TESS_WINDING_ODD,
            WindingRule::NonZero => TessWindingRule_TESS_WINDING_NONZERO,
            WindingRule::Positive => TessWindingRule_TESS_WINDING_POSITIVE,
            WindingRule::Negative => TessWindingRule_TESS_WINDING_NEGATIVE,
            WindingRule::AbsGeqTwo => TessWindingRule_TESS_WINDING_ABS_GEQ_TWO,
        }
    }
}

#[derive(Clone, Copy)]
pub enum ElementType {
    Polygons,
    ConnectedPolygons,
    BoundaryPolygons,
}

impl From<ElementType> for TessElementType {
    fn from(value: ElementType) -> Self {
        match value {
            ElementType::Polygons => TessElementType_TESS_POLYGONS,
            ElementType::ConnectedPolygons => TessElementType_TESS_CONNECTED_POLYGONS,
            ElementType::BoundaryPolygons => TessElementType_TESS_BOUNDARY_CONTOURS,
        }
    }
}

#[derive(Clone, Copy)]
pub enum TesselatorOption {
    ConstrainedDelaunayTriangulation,
    ReverseContour,
}

impl From<TesselatorOption> for TessOption {
    fn from(value: TesselatorOption) -> Self {
        match value {
            TesselatorOption::ConstrainedDelaunayTriangulation => {
                TessOption_TESS_CONSTRAINED_DELAUNAY_TRIANGULATION
            }
            TesselatorOption::ReverseContour => TessOption_TESS_REVERSE_CONTOURS,
        }
    }
}

pub type Float = TESSreal;
pub struct Tesselator {
    _tess: *mut TESStesselator,
}

impl Tesselator {
    pub fn new() -> Self {
        let _tess = unsafe { tessNewTess(std::ptr::null_mut()) };
        Self { _tess: _tess }
    }

    pub fn set_option(&self, option: TesselatorOption, value: i32) {
        let tess_option: TessOption = option.into();
        unsafe { tessSetOption(self._tess, tess_option.try_into().unwrap(), value) }
    }

    pub fn add_contour(&self, points: Vec<[Float; 2]>) {
        let stride: c_int = mem::size_of::<Float>().try_into().unwrap();
        let pointer = points.as_ptr();
        let ptr_to_void: *const c_void = pointer as *const c_void;
        let size: c_int = 2;
        let count: c_int = points.len().try_into().unwrap();
        unsafe { tessAddContour(self._tess, size, ptr_to_void, stride * size, count) }
    }

    pub fn tesselate(&self, winding_rule: WindingRule) -> Option<(Vec<[Float; 2]>, Vec<u32>)> {
        let element_type: ElementType = ElementType::Polygons;
        let tess_winding_rule: TessWindingRule = winding_rule.into();
        let tess_element_type: TessElementType = element_type.into();
        let poly_size: c_int = 3;
        let vertex_size: c_int = 2;
        let result = unsafe {
            tessTesselate(
                self._tess,
                tess_winding_rule.try_into().unwrap(),
                tess_element_type.try_into().unwrap(),
                poly_size,
                vertex_size,
                std::ptr::null(),
            )
        };

        if result == 0 {
            return None;
        }
        let vertices: *const f32;
        let vertex_count: i32;
        let elements: *const i32;
        let element_count: i32;
        unsafe {
            vertices = tessGetVertices(self._tess);
            vertex_count = tessGetVertexCount(self._tess);
            elements = tessGetElements(self._tess);
            element_count = tessGetElementCount(self._tess);
        }

        let mut indices_out: Vec<u32> = Vec::new();
        let tess_undef = -1;
        let poly_size = 3;
        for i in 0..element_count {
            let p = unsafe { elements.offset((i * poly_size).try_into().unwrap()) };
            for j in 0..poly_size {
                let idx = unsafe { *p.offset(j as isize) };
                if idx != tess_undef {
                    indices_out.push(idx as u32);
                }
            }
        }

        let vertex_size = 2;
        let mut vertices_out: Vec<[Float; 2]> = Vec::new();
        for i in 0..vertex_count {
            unsafe {
                let x = vertices.offset(i as isize * vertex_size);
                let y = vertices.offset(i as isize * vertex_size + 1);
                vertices_out.push([*x, *y]);
            }
        }

        return Some((vertices_out, indices_out));
    }
}

impl Drop for Tesselator {
    fn drop(&mut self) {
        unsafe { tessDeleteTess(self._tess) }
    }
}
