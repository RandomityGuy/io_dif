use crate::io::Writable;
use crate::io::*;
use crate::types::*;
use bytes::{Buf, BufMut};
use dif_derive::{Readable, Writable};

#[derive(Debug, Clone)]
pub struct StaticMesh {
    pub primitives: Vec<Primitive>,
    pub indices: Vec<u16>,
    pub vertexes: Vec<Point3F>,
    pub normals: Vec<Point3F>,
    pub diffuse_uvs: Vec<Point2F>,
    pub lightmap_uvs: Vec<Point2F>,

    pub base_material_list: Option<MaterialList>,

    pub has_solid: u8,
    pub has_translucency: u8,
    pub bounds: BoxF,
    pub transform: MatrixF,
    pub scale: Point3F,
}

#[derive(Debug, Readable, Writable, Clone)]
pub struct Primitive {
    pub alpha: u8,
    pub tex_s: u32,
    pub tex_t: u32,
    pub diffuse_index: i32,
    pub light_map_index: i32,
    pub start: u32,
    pub count: u32,
    pub light_map_equation_x: PlaneF,
    pub light_map_equation_y: PlaneF,
    pub light_map_offset: Point2I,
    pub light_map_size: Point2I,
}

#[derive(Debug, Clone)]
pub struct Material {
    pub flags: u32,
    pub reflectance_map: u32,
    pub bump_map: u32,
    pub detail_map: u32,
    pub light_map: u32,
    pub detail_scale: u32,
    pub reflection_amount: u32,
    pub diffuse_bitmap: Option<PNG>,
}

#[derive(Debug, Clone)]
pub struct MaterialList {
    pub materials: Vec<Material>,
}

impl Readable<StaticMesh> for StaticMesh {
    fn read(from: &mut dyn Buf, version: &mut Version) -> DifResult<Self> {
        let primitives = Vec::<Primitive>::read(from, version)?;
        let indices = Vec::<u16>::read(from, version)?;
        let vertexes = Vec::<Point3F>::read(from, version)?;
        let normals = Vec::<Point3F>::read(from, version)?;
        let diffuse_uvs = Vec::<Point2F>::read(from, version)?;
        let lightmap_uvs = Vec::<Point2F>::read(from, version)?;

        let base_material_list = if u8::read(from, version)? == 0 {
            None
        } else {
            Some(MaterialList::read(from, version)?)
        };

        let has_solid = u8::read(from, version)?;
        let has_translucency = u8::read(from, version)?;
        let bounds = BoxF::read(from, version)?;
        let transform = MatrixF::read(from, version)?;
        let scale = Point3F::read(from, version)?;

        Ok(StaticMesh {
            primitives,
            indices,
            vertexes,
            normals,
            diffuse_uvs,
            lightmap_uvs,
            base_material_list,
            has_solid,
            has_translucency,
            bounds,
            transform,
            scale,
        })
    }
}

impl Writable<StaticMesh> for StaticMesh {
    fn write(&self, _to: &mut dyn BufMut, _version: &Version) -> DifResult<()> {
        unimplemented!()
    }
}

impl Readable<MaterialList> for MaterialList {
    fn read(_from: &mut dyn Buf, _version: &mut Version) -> DifResult<Self> {
        // Yikes
        unimplemented!()
    }
}
