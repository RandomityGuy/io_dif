use crate::io::*;
use crate::types::*;
use bytes::{Buf, BufMut};
use dif_derive::{Readable, Writable};

#[derive(Debug, Readable, Writable)]
pub struct AISpecialNode {
    pub name: String,
    pub position: Point3F,
}
