use crate::io::*;
use crate::types::*;
use bytes::{Buf, BufMut};
use dif_derive::{Readable, Writable};

#[derive(Debug, Readable, Writable)]
pub struct ForceField {
    pub version: u32,
    pub name: String,
    pub triggers: Vec<String>,
    pub bounding_box: BoxF,
    pub bounding_sphere: SphereF,
    pub normals: Vec<Point3F>,
    pub planes: Vec<Plane>,
    pub bsp_nodes: Vec<BSPNode>,
    pub bsp_solid_leaves: Vec<BSPSolidLeaf>,
    pub indices: Vec<u32>,
    pub surfaces: Vec<Surface>,
    pub solid_leaf_surfaces: Vec<u32>,
    pub color: ColorI,
}

#[derive(Debug, Readable, Writable)]
pub struct Plane {
    pub normal_index: u32,
    pub plane_distance: f32,
}

#[derive(Debug, Readable, Writable)]
pub struct BSPNode {
    pub front_index: u16,
    pub back_index: u16,
}

#[derive(Debug, Readable, Writable)]
pub struct BSPSolidLeaf {
    pub surface_index: u32,
    pub surface_count: u16,
}

#[derive(Debug, Readable, Writable)]
pub struct Surface {
    pub winding_start: u32,
    pub winding_count: u8,
    pub plane_index: u16,
    pub surface_flags: u8,
    pub fan_mask: u32,
}
