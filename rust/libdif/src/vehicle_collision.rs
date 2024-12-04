use crate::io::*;
use crate::types::*;
use bytes::{Buf, BufMut};
use dif_derive::{Readable, Writable};

#[derive(Debug, Readable, Writable)]
pub struct VehicleCollision {
    pub version: u32,
    pub convex_hulls: Vec<ConvexHull>,
    pub convex_hull_emit_string_characters: Vec<u8>,
    pub hull_indices: Vec<u32>,
    pub hull_plane_indices: Vec<u16>,
    pub hull_emit_string_indices: Vec<u32>,
    pub hull_surface_indices: Vec<u32>,
    pub poly_list_plane_indices: Vec<u16>,
    pub poly_list_point_indices: Vec<u32>,
    pub poly_list_string_characters: Vec<u8>,
    pub null_surfaces: Vec<NullSurface>,
    pub points: Vec<Point3F>,
    pub planes: Vec<PlaneF>,
    pub windings: Vec<u32>,
    pub winding_indices: Vec<WindingIndex>,
}

#[derive(Debug, Readable, Writable)]
pub struct ConvexHull {
    pub hull_start: u32,
    pub hull_count: u16,
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub min_z: f32,
    pub max_z: f32,
    pub surface_start: u32,
    pub surface_count: u16,
    pub plane_start: u32,
    pub poly_list_plane_start: u32,
    pub poly_list_point_start: u32,
    pub poly_list_string_start: u32,
}

#[derive(Debug, Readable, Writable)]
pub struct NullSurface {
    pub winding_start: u32,
    pub plane_index: u16,
    pub surface_flags: u8,
    pub winding_count: u32,
}

#[derive(Debug, Readable, Writable)]
pub struct WindingIndex {
    pub winding_start: u32,
    pub winding_count: u32,
}
