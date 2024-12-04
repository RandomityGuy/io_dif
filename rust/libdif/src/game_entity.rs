use crate::io::*;
use crate::types::*;
use bytes::{Buf, BufMut};
use dif_derive::{Readable, Writable};

#[derive(Debug, Readable, Writable)]
pub struct GameEntity {
    pub datablock: String,
    pub game_class: String,
    pub position: Point3F,
    pub properties: Dictionary,
}
