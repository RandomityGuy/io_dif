use crate::io::*;
use crate::io::{Readable, Writable};
use crate::types::*;
use bytes::{Buf, BufMut};
use dif_derive::{Readable, Writable};

#[derive(Debug)]
pub struct Trigger {
    pub name: String,
    pub datablock: String,
    pub properties: Dictionary,
    pub polyhedron: Polyhedron,
    pub offset: Point3F,
}

#[derive(Debug, Readable, Writable)]
pub struct Polyhedron {
    pub point_list: Vec<Point3F>,
    pub plane_list: Vec<PlaneF>,
    pub edge_list: Vec<PolyhedronEdge>,
}

#[derive(Debug, Readable, Writable)]
pub struct PolyhedronEdge {
    pub face0: u32,
    pub face1: u32,
    pub vertex0: u32,
    pub vertex1: u32,
}

impl Readable<Trigger> for Trigger {
    fn read(from: &mut dyn Buf, version: &mut Version) -> DifResult<Self> {
        Ok(Trigger {
            name: String::read(from, version)?,
            datablock: String::read(from, version)?,
            properties: Dictionary::read(from, version)?,
            polyhedron: Polyhedron::read(from, version)?,
            offset: Point3F::read(from, version)?,
        })
    }
}

impl Writable<Trigger> for Trigger {
    fn write(&self, to: &mut dyn BufMut, version: &Version) -> DifResult<()> {
        self.name.write(to, version)?;
        self.datablock.write(to, version)?;
        if version.engine == EngineVersion::MBG {
            self.properties.write(to, version)?;
        }
        self.polyhedron.write(to, version)?;
        self.offset.write(to, version)?;
        Ok(())
    }
}
