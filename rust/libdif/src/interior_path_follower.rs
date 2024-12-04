use crate::io::*;
use crate::types::*;
use bytes::{Buf, BufMut};
use dif_derive::{Readable, Writable};

#[derive(Debug, Readable, Writable)]
pub struct InteriorPathFollower {
    pub name: String,
    pub datablock: String,
    pub interior_res_index: u32,
    pub offset: Point3F,
    pub properties: Dictionary,
    pub trigger_ids: Vec<u32>,
    pub way_points: Vec<WayPoint>,
    pub total_ms: u32,
}

#[derive(Debug, Readable, Writable, Copy, Clone)]
pub struct WayPoint {
    pub position: Point3F,
    pub rotation: QuatF,
    pub ms_to_next: u32,
    pub smoothing_type: u32,
}
