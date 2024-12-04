use crate::io::*;
use crate::io::{Readable, Writable};
use crate::static_mesh::StaticMesh;
use crate::sub_object::SubObject;
use crate::types::*;
use bytes::{Buf, BufMut};
use dif_derive::{Readable, Writable};
use std::io::Cursor;
use typed_ints::TypedInt;

typed_int!(PointIndex, _PointIndex, u32);
typed_int!(SurfaceIndex, _SurfaceIndex, u16);
typed_int!(NullSurfaceIndex, _NullSurfaceIndex, u16);
typed_int!(SolidLeafSurfaceIndex, _SolidLeafSurfaceIndex, u32);
typed_int!(StaticMeshIndex, _StaticMeshIndex, u32);
typed_int!(PortalIndex, _PortalIndex, u16);
typed_int!(NormalIndex, _NormalIndex, u16);
typed_int!(LMapIndex, _LMapIndex, u32);
typed_int!(PlaneIndex, _PlaneIndex, u16);
typed_int!(EmitStringIndex, _EmitStringIndex, u32);
typed_int!(CoordBinIndex, _CoordBinIndex, u32);
typed_int!(TexMatrixIndex, _TexMatrixIndex, u32);
typed_int!(ConvexHullIndex, _ConvexHullIndex, u16);
typed_int!(ZoneIndex, _ZoneIndex, u16);
typed_int!(WindingIndexIndex, _WindingIndexIndex, u32);
typed_int!(TextureIndex, _TextureIndex, u16);
typed_int!(TexGenIndex, _TexGenIndex, u32);
typed_int!(HullSurfaceIndex, _HullSurfaceIndex, u32);
typed_int!(HullPointIndex, _HullPointIndex, u32);
typed_int!(HullPlaneIndex, _HullPlaneIndex, u32);
typed_int!(PolyListPlaneIndex, _PolyListPlaneIndex, u32);
typed_int!(PolyListPointIndex, _PolyListPointIndex, u32);
typed_int!(PolyListStringIndex, _PolyListStringIndex, u32);

#[derive(Debug, Clone)]
pub struct Interior {
    pub detail_level: u32,
    pub min_pixels: u32,
    pub bounding_box: BoxF,
    pub bounding_sphere: SphereF,
    pub has_alarm_state: u8,
    pub num_light_state_entries: u32,

    pub normals: Vec<Point3F>,
    pub planes: Vec<Plane>,
    pub points: Vec<Point3F>,
    pub point_visibilities: Vec<u8>,
    pub tex_gen_eqs: Vec<TexGenEq>,
    pub bsp_nodes: Vec<BSPNode>,
    pub bsp_solid_leaves: Vec<BSPSolidLeaf>,

    pub material_names: Vec<String>,
    pub indices: Vec<PointIndex>,
    pub winding_indices: Vec<WindingIndex>,
    pub edges: Vec<Edge>,
    pub zones: Vec<Zone>,
    pub zone_surfaces: Vec<SurfaceIndex>,
    pub zone_static_meshes: Vec<StaticMeshIndex>,
    pub zone_portal_lists: Vec<PortalIndex>,
    pub portals: Vec<Portal>,
    pub surfaces: Vec<Surface>,
    pub edge2s: Vec<Edge2>,
    pub normal2s: Vec<Point3F>,
    pub normal_indices: Vec<NormalIndex>,
    pub normal_lmap_indices: Vec<LMapIndex>,
    pub alarm_lmap_indices: Vec<LMapIndex>,
    pub null_surfaces: Vec<NullSurface>,
    pub light_maps: Vec<LightMap>,
    pub solid_leaf_surfaces: Vec<PossiblyNullSurfaceIndex>,
    pub animated_lights: Vec<AnimatedLight>,
    pub light_states: Vec<LightState>,
    pub state_datas: Vec<StateData>,
    pub state_data_buffers: Vec<StateData>,

    pub flags: u32,

    pub name_buffer_characters: Vec<u8>,

    pub sub_objects: Vec<SubObject>,

    pub convex_hulls: Vec<ConvexHull>,
    pub convex_hull_emit_string_characters: Vec<u8>,
    pub hull_indices: Vec<PointIndex>,
    pub hull_plane_indices: Vec<PlaneIndex>,
    pub hull_emit_string_indices: Vec<EmitStringIndex>,
    pub hull_surface_indices: Vec<PossiblyNullSurfaceIndex>,
    pub poly_list_plane_indices: Vec<PlaneIndex>,
    pub poly_list_point_indices: Vec<PointIndex>,
    pub poly_list_string_characters: Vec<u8>,
    pub coord_bins: Vec<CoordBin>,
    pub coord_bin_indices: Vec<ConvexHullIndex>,

    pub coord_bin_mode: u32,
    pub base_ambient_color: ColorI,
    pub alarm_ambient_color: ColorI,

    pub static_meshes: Vec<StaticMesh>,
    pub tex_normals: Vec<Point3F>,
    pub tex_matrices: Vec<TexMatrix>,
    pub tex_matrix_indices: Vec<TexMatrixIndex>,

    pub extended_light_map_data: u32,
    pub light_map_border_size: u32,
}

#[derive(Debug, Readable, Writable, Clone)]
pub struct Plane {
    pub normal_index: NormalIndex,
    pub plane_distance: f32,
}

#[derive(Debug, Readable, Writable, Clone)]
pub struct TexGenEq {
    pub plane_x: PlaneF,
    pub plane_y: PlaneF,
}

#[derive(Debug, Clone)]
pub struct BSPIndex {
    pub index: u32,
    pub leaf: bool,
    pub solid: bool,
}

#[derive(Debug, Clone)]
pub struct BSPNode {
    pub plane_index: PlaneIndex,
    pub front_index: BSPIndex,
    pub back_index: BSPIndex,
}

#[derive(Debug, Readable, Writable, Clone)]
pub struct BSPSolidLeaf {
    pub surface_index: SolidLeafSurfaceIndex,
    pub surface_count: u16,
}

#[derive(Debug, Readable, Writable, Clone)]
pub struct WindingIndex {
    pub winding_start: PointIndex,
    pub winding_count: u32,
}

#[derive(Debug, Readable, Writable, Clone)]
pub struct Edge {
    pub point_index0: i32,
    pub point_index1: i32,
    pub surface_index0: i32,
    pub surface_index1: i32,
}

#[derive(Debug, Clone)]
pub struct Zone {
    pub portal_start: PortalIndex,
    pub portal_count: u16,
    pub surface_start: u32,
    pub surface_count: u32,
    pub static_mesh_start: StaticMeshIndex,
    pub static_mesh_count: u32,
    pub flags: u16,
}

#[derive(Debug, Readable, Writable, Clone)]
pub struct Portal {
    pub plane_index: PlaneIndex,
    pub tri_fan_count: u16,
    pub tri_fan_start: WindingIndexIndex,
    pub zone_front: ZoneIndex,
    pub zone_back: ZoneIndex,
}

#[derive(Debug, Clone)]
pub struct LightMap {
    pub light_map: PNG,
    pub light_dir_map: Option<PNG>,
    pub keep_light_map: u8,
}

#[derive(Debug, Readable, Writable, Clone)]
pub struct SurfaceLightMap {
    pub final_word: u16,
    pub tex_gen_x_distance: f32,
    pub tex_gen_y_distance: f32,
}

bitflags! {
    pub struct SurfaceFlags: u8 {
        const DETAIL = 0b1;
        const AMBIGUOUS = 0b10;
        const ORPHAN = 0b100;
        const SHARED_LIGHT_MAPS = 0b1000;
        const OUTSIDE_VISIBLE = 0b10000;
    }
}

#[derive(Debug, Clone)]
pub struct Surface {
    pub winding_start: WindingIndexIndex,
    pub winding_count: u32,
    pub plane_index: PlaneIndex,
    pub plane_flipped: bool,
    pub texture_index: TextureIndex,
    pub tex_gen_index: TexGenIndex,
    pub surface_flags: SurfaceFlags,
    pub fan_mask: u32,
    pub light_map: SurfaceLightMap,
    pub light_count: u16,
    pub light_state_info_start: u32,
    pub map_offset_x: u32,
    pub map_offset_y: u32,
    pub map_size_x: u32,
    pub map_size_y: u32,
    pub brush_id: u32,
}

#[derive(Debug, Clone)]
pub enum PossiblyNullSurfaceIndex {
    Null(NullSurfaceIndex),
    NonNull(SurfaceIndex),
}

#[derive(Debug, Clone)]
pub struct Edge2 {
    pub vertices: [u32; 2],
    pub normals: [u32; 2],
    pub faces: [u32; 2],
}

#[derive(Debug, Clone)]
pub struct NullSurface {
    pub winding_start: WindingIndexIndex,
    pub plane_index: PlaneIndex,
    pub surface_flags: SurfaceFlags,
    pub winding_count: u8,
}

#[derive(Debug, Readable, Writable, Clone)]
pub struct AnimatedLight {
    pub name_index: u32,
    pub state_index: u32,
    pub state_count: u16,
    pub flags: u16,
    pub duration: u32,
}

#[derive(Debug, Readable, Writable, Clone)]
pub struct LightState {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub active_time: u32,
    pub data_index: u32,
    pub data_count: u16,
}

#[derive(Debug, Readable, Writable, Clone)]
pub struct StateData {
    pub surface_index: u32,
    pub map_index: u32,
    pub light_state_index: u16,
}

#[derive(Debug, Clone)]
pub struct ConvexHull {
    pub hull_start: HullPointIndex, //HullEmitStringIndex
    pub hull_count: u16,
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub min_z: f32,
    pub max_z: f32,
    pub surface_start: HullSurfaceIndex,
    pub surface_count: u16,
    pub plane_start: HullPlaneIndex,
    pub poly_list_plane_start: PolyListPlaneIndex,
    pub poly_list_point_start: PolyListPointIndex,
    pub poly_list_string_start: PolyListStringIndex,
    pub static_mesh: u8,
}

#[derive(Debug, Readable, Writable, Clone)]
pub struct CoordBin {
    pub bin_start: CoordBinIndex,
    pub bin_count: u32,
}

#[derive(Debug, Readable, Writable, Clone)]
pub struct TexMatrix {
    pub t: i32,
    pub n: i32,
    pub b: i32,
}

impl Readable<Interior> for Interior {
    fn read(from: &mut dyn Buf, version: &mut Version) -> DifResult<Self> {
        version.interior = u32::read(from, version)?;
        if version.interior > 14 {
            return Err(DifError::from("Stuff"));
        }

        let detail_level = u32::read(from, version)?;
        let min_pixels = u32::read(from, version)?;
        let bounding_box = BoxF::read(from, version)?;
        let bounding_sphere = SphereF::read(from, version)?;
        let has_alarm_state = u8::read(from, version)?;
        let num_light_state_entries = u32::read(from, version)?;
        let normals = Vec::<Point3F>::read(from, version)?;
        let planes = Vec::<Plane>::read(from, version)?;
        let points = Vec::<Point3F>::read(from, version)?;

        let point_visibilities = if version.interior != 4 {
            Vec::<u8>::read(from, version)?
        } else {
            //Probably defaulted to FF but uncertain
            vec![]
        };

        let tex_gen_eqs = Vec::<TexGenEq>::read(from, version)?;
        let bsp_nodes = Vec::<BSPNode>::read(from, version)?;
        let bsp_solid_leaves = Vec::<BSPSolidLeaf>::read(from, version)?;

        version.material_list = u8::read(from, version)?;
        let material_names = Vec::<String>::read(from, version)?;

        let indices =
            read_vec::<PointIndex, u16>(from, version, |p, _| p, |x| PointIndex::new(x as _))?;
        let winding_indices = Vec::<WindingIndex>::read(from, version)?;
        let edges = if version.interior >= 12 {
            Vec::<Edge>::read(from, version)?
        } else {
            vec![]
        };
        let zones = Vec::<Zone>::read(from, version)?;
        let zone_surfaces =
            read_vec::<SurfaceIndex, u16>(from, version, |_, _| false, |x| SurfaceIndex::new(x))?;
        let zone_static_meshes = if version.interior >= 12 {
            Vec::<StaticMeshIndex>::read(from, version)?
        } else {
            vec![]
        };
        let zone_portal_lists =
            read_vec::<PortalIndex, u16>(from, version, |_, _| false, |x| PortalIndex::new(x))?;
        let portals = Vec::<Portal>::read(from, version)?;

        // Buf doesn't support seeking, so we have to

        let mut from2 = Cursor::new(from.bytes());

        let surfaces = if let Ok(surfaces) = read_vec_fn(&mut from2, version, |from, version| {
            Surface::read(
                from,
                version,
                indices.len(),
                planes.len(),
                material_names.len(),
                tex_gen_eqs.len(),
            )
        }) {
            //Read successfully as a T3D/TGEA dif
            if version.engine == EngineVersion::Unknown {
                version.engine = EngineVersion::TGEA;
            }

            let remain = from.remaining() - from2.remaining();
            from.advance(remain);

            Ok(surfaces)
        } else {
            //Not a TGE/TGEA dif, probably MBG. Not really sure if we can detect the
            // difference here, needs more error checking.
            if version.engine == EngineVersion::Unknown {
                version.engine = EngineVersion::MBG;
            }

            if version.interior != 0 {
                Err(DifError::from("Invalid version for MBG interior"))
            } else {
                //Ok so we failed reading, it's *probably* a TGE interior. Let's try
                // to read it as a TGE interior.

                read_vec_fn(from, version, |from, version| {
                    Surface::read(
                        from,
                        version,
                        indices.len(),
                        planes.len(),
                        material_names.len(),
                        tex_gen_eqs.len(),
                    )
                })
            }
        }?;

        //Edge data from MBU levels and beyond in some cases
        let edge2s = if version.interior >= 2 && version.interior <= 5 {
            Vec::<Edge2>::read(from, version)?
        } else {
            vec![]
        };

        //v4 has some extra points and indices, they're probably used with the edges
        // but I have no idea

        //Extra normals used in reading the edges?
        let normal2s = if version.interior >= 4 && version.interior <= 5 {
            Vec::<Point3F>::read(from, version)?
        } else {
            vec![]
        };

        //Looks like indices of some sort, can't seem to make them out though

        //Unlike anywhere else, these actually take the param into account.
        // If it's read2 and param == 0, then they use U8s, if param == 1, they use U16s
        // Not really sure why, haven't seen this anywhere else.
        let normal_indices = if version.interior >= 4 && version.interior <= 5 {
            read_vec::<NormalIndex, u8>(
                from,
                version,
                |alt, p| alt && p == 0,
                |x| NormalIndex::new(x as _),
            )?
        } else {
            vec![]
        };

        let normal_lmap_indices = if version.interior >= 13 {
            //These are 32-bit values in v13 and up
            Vec::<LMapIndex>::read(from, version)?
        } else {
            //Normally they're just 8
            read_vec::<LMapIndex, u8>(from, version, |_, _| true, |x| LMapIndex::new(x as _))?
        };
        let alarm_lmap_indices = if version.interior >= 13 {
            Vec::<LMapIndex>::read(from, version)?
        } else if version.interior != 4 {
            read_vec::<LMapIndex, u8>(from, version, |_, _| true, |x| LMapIndex::new(x as _))?
        } else {
            // Not included in version 4
            vec![]
        };

        let null_surfaces = Vec::<NullSurface>::read(from, version)?;

        //Also found in 0, 2, 3, 14
        let light_maps = if version.interior != 4 {
            Vec::<LightMap>::read(from, version)?
        } else {
            vec![]
        };
        if light_maps.len() > 0 && version.engine == EngineVersion::MBG {
            version.engine = EngineVersion::TGE;
        }
        let solid_leaf_surfaces = read_vec::<PossiblyNullSurfaceIndex, u16>(
            from,
            version,
            |alt, _| alt,
            |x| PossiblyNullSurfaceIndex::from(x as u32),
        )?;
        let animated_lights = Vec::<AnimatedLight>::read(from, version)?;
        let light_states = Vec::<LightState>::read(from, version)?;

        //Yet more things found in 0, 2, 3, 14
        let state_datas = if version.interior != 4 {
            Vec::<StateData>::read(from, version)?
        } else {
            vec![]
        };

        //State datas have the flags field written right after the vector size,
        // and THEN the data, just to make things confusing. So we need yet another
        // read method for this.
        let (state_data_buffers, flags) = if version.interior != 4 {
            read_vec_extra::<StateData, u32>(from, version, |from, version| {
                u32::read(from, version)
            })?
        } else {
            (vec![], 0)
        };

        let name_buffer_characters = if version.interior != 4 {
            Vec::<u8>::read(from, version)?
        } else {
            vec![]
        };

        let sub_objects = if version.interior != 4 {
            Vec::<SubObject>::read(from, version)?
        } else {
            vec![]
        };

        let convex_hulls = Vec::<ConvexHull>::read(from, version)?;
        let convex_hull_emit_string_characters = Vec::<u8>::read(from, version)?;

        //-------------------------------------------------------------------------
        // Lots of index lists here that have U16 or U32 versions based on loop2.
        // The actual bytes of the interior have 0x80s at the ends (negative bit)
        // which seems to specify that these take a smaller type. They managed to
        // save ~50KB/interior, but was it worth the pain?
        //
        // Also fun fact: the U16 lists have literally no reason for the 0x80, as
        // they're already using U16s. However, GG still puts them in.
        //-------------------------------------------------------------------------

        let hull_indices =
            read_vec::<PointIndex, u16>(from, version, |alt, _| alt, |x| PointIndex::new(x as _))?;
        let hull_plane_indices =
            read_vec::<PlaneIndex, u16>(from, version, |_, _| true, |x| PlaneIndex::new(x as _))?;
        let hull_emit_string_indices = read_vec::<EmitStringIndex, u16>(
            from,
            version,
            |alt, _| alt,
            |x| EmitStringIndex::new(x as _),
        )?;
        let hull_surface_indices = read_vec::<PossiblyNullSurfaceIndex, u16>(
            from,
            version,
            |alt, _| alt,
            |x| PossiblyNullSurfaceIndex::from(x as u32),
        )?;
        let poly_list_plane_indices =
            read_vec::<PlaneIndex, u16>(from, version, |_, _| true, |x| PlaneIndex::new(x as _))?;
        let poly_list_point_indices =
            read_vec::<PointIndex, u16>(from, version, |alt, _| alt, |x| PointIndex::new(x as _))?;

        //Not sure if this should be a read_as, but I haven't seen any evidence
        // of needing that for U8 lists.
        let poly_list_string_characters = Vec::<u8>::read(from, version)?;

        let mut coord_bins: Vec<CoordBin> = Vec::with_capacity(256);
        for _ in 0..256 {
            coord_bins.push(CoordBin::read(from, version)?);
        }

        let coord_bin_indices = read_vec::<ConvexHullIndex, u16>(
            from,
            version,
            |_, _| true,
            |x| ConvexHullIndex::new(x as _),
        )?;
        let coord_bin_mode = u32::read(from, version)?;

        //All of this is missing in v4 as well. Saves no space.
        let base_ambient_color = if version.interior != 4 {
            ColorI::read(from, version)?
        } else {
            ColorI {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            }
        };
        let alarm_ambient_color = if version.interior != 4 {
            ColorI::read(from, version)?
        } else {
            ColorI {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            }
        };
        let static_meshes = if version.interior >= 10 {
            Vec::<StaticMesh>::read(from, version)?
        } else {
            vec![]
        };
        let tex_normals = if version.interior >= 11 {
            Vec::<Point3F>::read(from, version)?
        } else if version.interior != 4 {
            let _ = u32::read(from, version)?;
            vec![]
        } else {
            vec![]
        };
        let tex_matrices = if version.interior >= 11 {
            Vec::<TexMatrix>::read(from, version)?
        } else if version.interior != 4 {
            let _ = u32::read(from, version)?;
            vec![]
        } else {
            vec![]
        };
        let tex_mat_indices = if version.interior >= 11 {
            Vec::<TexMatrixIndex>::read(from, version)?
        } else if version.interior != 4 {
            let _ = u32::read(from, version)?;
            vec![]
        } else {
            vec![]
        };
        let extended_light_map_data = if version.interior != 4 {
            u32::read(from, version)?
        } else {
            0
        };
        let light_map_border_size = if extended_light_map_data != 0 {
            let size = u32::read(from, version)?;
            let _ = u32::read(from, version)?;
            size
        } else {
            0
        };

        Ok(Interior {
            detail_level,
            min_pixels,
            bounding_box,
            bounding_sphere,
            has_alarm_state,
            num_light_state_entries,
            normals,
            planes,
            points,
            point_visibilities,
            tex_gen_eqs,
            bsp_nodes,
            bsp_solid_leaves,
            material_names,
            indices,
            winding_indices,
            edges,
            zones,
            zone_surfaces,
            zone_static_meshes,
            zone_portal_lists,
            portals,
            surfaces,
            edge2s,
            normal2s,
            normal_indices,
            normal_lmap_indices,
            alarm_lmap_indices,
            null_surfaces,
            light_maps,
            solid_leaf_surfaces,
            animated_lights,
            light_states,
            state_datas,
            state_data_buffers,
            flags,
            name_buffer_characters,
            sub_objects,
            convex_hulls,
            convex_hull_emit_string_characters,
            hull_indices,
            hull_plane_indices,
            hull_emit_string_indices,
            hull_surface_indices,
            poly_list_plane_indices,
            poly_list_point_indices,
            poly_list_string_characters,
            coord_bins,
            coord_bin_indices,
            coord_bin_mode,
            base_ambient_color,
            alarm_ambient_color,
            static_meshes,
            tex_normals,
            tex_matrices,
            tex_matrix_indices: tex_mat_indices,
            extended_light_map_data,
            light_map_border_size,
        })
    }
}

impl Writable<Interior> for Interior {
    fn write(&self, to: &mut dyn BufMut, version: &Version) -> DifResult<()> {
        version.interior.write(to, version)?;
        self.detail_level.write(to, version)?;
        self.min_pixels.write(to, version)?;
        self.bounding_box.write(to, version)?;
        self.bounding_sphere.write(to, version)?;
        self.has_alarm_state.write(to, version)?;
        self.num_light_state_entries.write(to, version)?;
        self.normals.write(to, version)?;
        self.planes.write(to, version)?;
        self.points.write(to, version)?;
        if version.interior != 4 {
            self.point_visibilities.write(to, version)?;
        }
        self.tex_gen_eqs.write(to, version)?;
        self.bsp_nodes.write(to, version)?;
        self.bsp_solid_leaves.write(to, version)?;
        version.material_list.write(to, version)?;
        self.material_names.write(to, version)?;
        self.indices.write(to, version)?;
        self.winding_indices.write(to, version)?;
        if version.interior >= 12 {
            self.edges.write(to, version)?;
        }
        self.zones.write(to, version)?;
        self.zone_surfaces.write(to, version)?;
        if version.interior >= 12 {
            self.zone_static_meshes.write(to, version)?;
        }
        self.zone_portal_lists.write(to, version)?;
        self.portals.write(to, version)?;
        self.surfaces.write(to, version)?;
        if version.interior >= 2 && version.interior <= 5 {
            self.edge2s.write(to, version)?;
        }
        if version.interior >= 4 && version.interior <= 5 {
            self.normal2s.write(to, version)?;
        }
        if version.interior >= 4 && version.interior <= 5 {
            self.normal_indices.write(to, version)?;
        }
        if version.interior >= 13 {
            self.normal_lmap_indices.write(to, version)?;
        } else {
            write_vec_fn::<LMapIndex, u8>(&self.normal_lmap_indices, to, version, |i| {
                *i.inner() as u8
            })?;
        }
        if version.interior >= 13 {
            self.alarm_lmap_indices.write(to, version)?;
        } else if version.interior != 4 {
            write_vec_fn::<LMapIndex, u8>(&self.alarm_lmap_indices, to, version, |i| {
                *i.inner() as u8
            })?;
        }
        self.null_surfaces.write(to, version)?;
        if version.interior != 4 {
            self.light_maps.write(to, version)?;
        }
        self.solid_leaf_surfaces.write(to, version)?;
        self.animated_lights.write(to, version)?;
        self.light_states.write(to, version)?;
        if version.interior != 4 {
            self.state_datas.write(to, version)?;
        }
        if version.interior != 4 {
            write_vec_extra(&self.state_data_buffers, to, version, |to, version| {
                self.flags.write(to, version)
            })?;
        }
        if version.interior != 4 {
            self.name_buffer_characters.write(to, version)?;
        }
        if version.interior != 4 {
            self.sub_objects.write(to, version)?;
        }
        self.convex_hulls.write(to, version)?;
        self.convex_hull_emit_string_characters.write(to, version)?;
        self.hull_indices.write(to, version)?;
        self.hull_plane_indices.write(to, version)?;
        self.hull_emit_string_indices.write(to, version)?;
        self.hull_surface_indices.write(to, version)?;
        self.poly_list_plane_indices.write(to, version)?;
        self.poly_list_point_indices.write(to, version)?;
        self.poly_list_string_characters.write(to, version)?;

        for bin in &self.coord_bins {
            bin.write(to, version)?;
        }

        self.coord_bin_indices.write(to, version)?;
        self.coord_bin_mode.write(to, version)?;
        if version.interior != 4 {
            self.base_ambient_color.write(to, version)?;
        }
        if version.interior != 4 {
            self.alarm_ambient_color.write(to, version)?;
        }
        if version.interior >= 10 {
            self.static_meshes.write(to, version)?;
        }
        if version.interior >= 11 {
            self.tex_normals.write(to, version)?;
        } else if version.interior != 4 {
            0u32.write(to, version)?;
        }
        if version.interior >= 11 {
            self.tex_matrices.write(to, version)?;
        } else if version.interior != 4 {
            0u32.write(to, version)?;
        }
        if version.interior >= 11 {
            self.tex_matrix_indices.write(to, version)?;
        } else if version.interior != 4 {
            0u32.write(to, version)?;
        }
        if version.interior != 4 {
            self.extended_light_map_data.write(to, version)?;
            if self.extended_light_map_data != 0 {
                self.light_map_border_size.write(to, version)?;
                0u32.write(to, version)?;
            }
        }

        Ok(())
    }
}

impl BSPIndex {
    fn read_bspnode(from: &mut dyn Buf, version: &mut Version) -> DifResult<Self> {
        let mut leaf = false;
        let mut solid = false;
        let index = if version.interior >= 14 {
            let mut index = u32::read(from, version)?;
            if index & 0x80000 != 0 {
                index = index & !0x80000;
                leaf = true;
            }
            if index & 0x40000 != 0 {
                index = index & !0x40000;
                solid = true;
            }
            index
        } else {
            let mut index = u16::read(from, version)?;
            if index & 0x8000 != 0 {
                index = index & !0x8000;
                leaf = true;
            }
            if index & 0x4000 != 0 {
                index = index & !0x4000;
                solid = true;
            }
            index as u32
        };
        Ok(BSPIndex { index, leaf, solid })
    }

    fn write_bspnode(&self, to: &mut dyn BufMut, version: &Version) -> DifResult<()> {
        if version.interior >= 14 {
            let mut index = self.index;
            if self.leaf {
                index |= 0x80000;
            }
            if self.solid {
                index |= 0x40000;
            }
            index.write(to, version)?;
        } else {
            let mut index = self.index;
            if self.leaf {
                index |= 0x8000;
            }
            if self.solid {
                index |= 0x4000;
            }
            (index as u16).write(to, version)?;
        }
        Ok(())
    }
}

impl Readable<BSPNode> for BSPNode {
    fn read(from: &mut dyn Buf, version: &mut Version) -> DifResult<Self> {
        let plane_index = PlaneIndex::read(from, version)?;
        let front_index = BSPIndex::read_bspnode(from, version)?;
        let back_index = BSPIndex::read_bspnode(from, version)?;
        Ok(BSPNode {
            plane_index,
            front_index,
            back_index,
        })
    }
}

impl Writable<BSPNode> for BSPNode {
    fn write(&self, to: &mut dyn BufMut, version: &Version) -> DifResult<()> {
        self.plane_index.write(to, version)?;
        self.front_index.write_bspnode(to, version)?;
        self.back_index.write_bspnode(to, version)?;
        Ok(())
    }
}

impl Readable<Zone> for Zone {
    fn read(from: &mut dyn Buf, version: &mut Version) -> DifResult<Self> {
        let portal_start = PortalIndex::read(from, version)?;
        let portal_count = u16::read(from, version)?;
        let surface_start = u32::read(from, version)?;
        let surface_count = u32::read(from, version)?;
        let static_mesh_start = if version.interior >= 12 {
            StaticMeshIndex::read(from, version)?
        } else {
            StaticMeshIndex::new(0u32)
        };
        let static_mesh_count = if version.interior >= 12 {
            u32::read(from, version)?
        } else {
            0
        };
        Ok(Zone {
            portal_start,
            portal_count,
            surface_start,
            surface_count,
            static_mesh_start,
            static_mesh_count,
            flags: 0,
        })
    }
}

impl Writable<Zone> for Zone {
    fn write(&self, to: &mut dyn BufMut, version: &Version) -> DifResult<()> {
        self.portal_start.write(to, version)?;
        self.portal_count.write(to, version)?;
        self.surface_start.write(to, version)?;
        self.surface_count.write(to, version)?;
        if version.interior >= 12 {
            self.static_mesh_start.write(to, version)?;
        }
        if version.interior >= 12 {
            self.static_mesh_count.write(to, version)?;
        }
        Ok(())
    }
}

impl Readable<LightMap> for LightMap {
    fn read(from: &mut dyn Buf, version: &mut Version) -> DifResult<Self> {
        Ok(LightMap {
            light_map: PNG::read(from, version)?,
            light_dir_map: if version.is_tge() {
                None
            } else {
                Some(PNG::read(from, version)?)
            },
            keep_light_map: u8::read(from, version)?,
        })
    }
}

impl Writable<LightMap> for LightMap {
    fn write(&self, to: &mut dyn BufMut, version: &Version) -> DifResult<()> {
        self.light_map.write(to, version)?;
        if !version.is_tge() {
            if let Some(lm) = &self.light_dir_map {
                lm.write(to, version)?;
            }
        }
        self.keep_light_map.write(to, version)?;
        Ok(())
    }
}

// Not a Readable<Surface>
impl Surface {
    pub fn read(
        from: &mut dyn Buf,
        version: &mut Version,
        indices_len: usize,
        planes_len: usize,
        material_names_len: usize,
        tex_gen_eqs_len: usize,
    ) -> DifResult<Surface> {
        let winding_start = u32::read(from, version)?;
        let winding_count = if version.interior >= 13 {
            u32::read(from, version)?
        } else {
            u8::read(from, version)? as u32
        };
        if (winding_start + winding_count) as usize > indices_len {
            return Err(DifError::from("OOB"));
        }

        let mut plane_index = u16::read(from, version)?;
        let plane_flipped = plane_index >> 15 != 0;
        plane_index &= !0x8000;
        if plane_index as usize >= planes_len {
            return Err(DifError::from("OOB"));
        }

        let texture_index = u16::read(from, version)?;
        if texture_index as usize >= material_names_len {
            return Err(DifError::from("OOB"));
        }

        let tex_gen_index = u32::read(from, version)?;
        if tex_gen_index as usize >= tex_gen_eqs_len {
            return Err(DifError::from("OOB"));
        }

        let surface_flags =
            SurfaceFlags::from_bits(u8::read(from, version)?).ok_or_else(|| "Invalid flags")?;
        let fan_mask = u32::read(from, version)?;
        let light_map = SurfaceLightMap::read(from, version)?;
        let light_count = u16::read(from, version)?;
        let light_state_info_start = u32::read(from, version)?;

        let map_offset_x = if version.interior >= 13 {
            u32::read(from, version)?
        } else {
            u8::read(from, version)? as u32
        };
        let map_offset_y = if version.interior >= 13 {
            u32::read(from, version)?
        } else {
            u8::read(from, version)? as u32
        };
        let map_size_x = if version.interior >= 13 {
            u32::read(from, version)?
        } else {
            u8::read(from, version)? as u32
        };
        let map_size_y = if version.interior >= 13 {
            u32::read(from, version)?
        } else {
            u8::read(from, version)? as u32
        };

        let mut brush_id = 0;
        if !version.is_tge() {
            let _ = u8::read(from, version)?;
            if version.interior >= 2 && version.interior <= 5 {
                brush_id = u32::read(from, version)?;
            }
        }

        Ok(Surface {
            winding_start: WindingIndexIndex::new(winding_start),
            winding_count,
            plane_index: PlaneIndex::new(plane_index),
            plane_flipped,
            texture_index: TextureIndex::new(texture_index),
            tex_gen_index: TexGenIndex::new(tex_gen_index),
            surface_flags,
            fan_mask,
            light_map,
            light_count,
            light_state_info_start,
            map_offset_x,
            map_offset_y,
            map_size_x,
            map_size_y,
            brush_id,
        })
    }
}

impl Writable<Surface> for Surface {
    fn write(&self, to: &mut dyn BufMut, version: &Version) -> DifResult<()> {
        self.winding_start.write(to, version)?;
        if version.interior >= 13 {
            self.winding_count.write(to, version)?;
        } else {
            (self.winding_count as u8).write(to, version)?;
        }
        let plane_index = if self.plane_flipped {
            *self.plane_index.inner() | 0x8000
        } else {
            *self.plane_index.inner()
        };
        plane_index.write(to, version)?;

        self.texture_index.write(to, version)?;
        self.tex_gen_index.write(to, version)?;
        self.surface_flags.bits().write(to, version)?;
        self.fan_mask.write(to, version)?;
        self.light_map.write(to, version)?;
        self.light_count.write(to, version)?;
        self.light_state_info_start.write(to, version)?;

        if version.interior >= 13 {
            self.map_offset_x.write(to, version)?;
        } else {
            (self.map_offset_x as u8).write(to, version)?;
        }

        if version.interior >= 13 {
            self.map_offset_y.write(to, version)?;
        } else {
            (self.map_offset_y as u8).write(to, version)?;
        }

        if version.interior >= 13 {
            self.map_size_x.write(to, version)?;
        } else {
            (self.map_size_x as u8).write(to, version)?;
        }

        if version.interior >= 13 {
            self.map_size_y.write(to, version)?;
        } else {
            (self.map_size_y as u8).write(to, version)?;
        }

        if !version.is_tge() {
            0u8.write(to, version)?;
            if version.interior >= 2 && version.interior <= 5 {
                self.brush_id.write(to, version)?;
            }
        }

        Ok(())
    }
}

impl Readable<Edge2> for Edge2 {
    fn read(from: &mut dyn Buf, version: &mut Version) -> DifResult<Self> {
        Ok(Edge2 {
            vertices: [u32::read(from, version)?, u32::read(from, version)?],
            normals: [u32::read(from, version)?, u32::read(from, version)?],
            faces: if version.interior >= 3 {
                [u32::read(from, version)?, u32::read(from, version)?]
            } else {
                [0, 0]
            },
        })
    }
}

impl Writable<Edge2> for Edge2 {
    fn write(&self, to: &mut dyn BufMut, version: &Version) -> DifResult<()> {
        self.vertices[0].write(to, version)?;
        self.vertices[1].write(to, version)?;
        self.normals[0].write(to, version)?;
        self.normals[1].write(to, version)?;
        if version.interior >= 3 {
            self.faces[0].write(to, version)?;
            self.faces[1].write(to, version)?;
        }
        Ok(())
    }
}

impl Readable<NullSurface> for NullSurface {
    fn read(from: &mut dyn Buf, version: &mut Version) -> DifResult<Self> {
        Ok(NullSurface {
            winding_start: WindingIndexIndex::read(from, version)?,
            plane_index: PlaneIndex::read(from, version)?,
            surface_flags: SurfaceFlags::from_bits(u8::read(from, version)?)
                .ok_or_else(|| "Invalid flags")?,
            winding_count: if version.interior >= 13 {
                u32::read(from, version)? as u8
            } else {
                u8::read(from, version)?
            },
        })
    }
}

impl Writable<NullSurface> for NullSurface {
    fn write(&self, to: &mut dyn BufMut, version: &Version) -> DifResult<()> {
        self.winding_start.write(to, version)?;
        self.plane_index.write(to, version)?;
        self.surface_flags.bits().write(to, version)?;
        if version.interior >= 13 {
            (self.winding_count as u32).write(to, version)?;
        } else {
            self.winding_count.write(to, version)?;
        }
        Ok(())
    }
}

impl Readable<ConvexHull> for ConvexHull {
    fn read(from: &mut dyn Buf, version: &mut Version) -> DifResult<Self> {
        Ok(ConvexHull {
            hull_start: HullPointIndex::read(from, version)?,
            hull_count: u16::read(from, version)?,
            min_x: f32::read(from, version)?,
            max_x: f32::read(from, version)?,
            min_y: f32::read(from, version)?,
            max_y: f32::read(from, version)?,
            min_z: f32::read(from, version)?,
            max_z: f32::read(from, version)?,
            surface_start: HullSurfaceIndex::read(from, version)?,
            surface_count: u16::read(from, version)?,
            plane_start: HullPlaneIndex::read(from, version)?,
            poly_list_plane_start: PolyListPlaneIndex::read(from, version)?,
            poly_list_point_start: PolyListPointIndex::read(from, version)?,
            poly_list_string_start: PolyListStringIndex::read(from, version)?,
            static_mesh: if version.interior >= 12 {
                u8::read(from, version)?
            } else {
                0
            },
        })
    }
}

impl Writable<ConvexHull> for ConvexHull {
    fn write(&self, to: &mut dyn BufMut, version: &Version) -> DifResult<()> {
        self.hull_start.write(to, version)?;
        self.hull_count.write(to, version)?;
        self.min_x.write(to, version)?;
        self.max_x.write(to, version)?;
        self.min_y.write(to, version)?;
        self.max_y.write(to, version)?;
        self.min_z.write(to, version)?;
        self.max_z.write(to, version)?;
        self.surface_start.write(to, version)?;
        self.surface_count.write(to, version)?;
        self.plane_start.write(to, version)?;
        self.poly_list_plane_start.write(to, version)?;
        self.poly_list_point_start.write(to, version)?;
        self.poly_list_string_start.write(to, version)?;
        if version.interior >= 12 {
            self.static_mesh.write(to, version)?;
        }
        Ok(())
    }
}

impl From<u32> for PossiblyNullSurfaceIndex {
    fn from(index: u32) -> Self {
        if index & 0x80000000 == 0x80000000 {
            PossiblyNullSurfaceIndex::Null(NullSurfaceIndex::new((index & 0xFFFF) as _))
        } else {
            PossiblyNullSurfaceIndex::NonNull(SurfaceIndex::new((index & 0xFFFF) as _))
        }
    }
}

impl From<NullSurfaceIndex> for PossiblyNullSurfaceIndex {
    fn from(index: NullSurfaceIndex) -> Self {
        PossiblyNullSurfaceIndex::Null(index)
    }
}

impl From<SurfaceIndex> for PossiblyNullSurfaceIndex {
    fn from(index: SurfaceIndex) -> Self {
        PossiblyNullSurfaceIndex::NonNull(index)
    }
}

impl Readable<PossiblyNullSurfaceIndex> for PossiblyNullSurfaceIndex {
    fn read(from: &mut dyn Buf, version: &mut Version) -> DifResult<Self> {
        let index = u32::read(from, version)?;
        Ok(PossiblyNullSurfaceIndex::from(index))
    }
}

impl Writable<PossiblyNullSurfaceIndex> for PossiblyNullSurfaceIndex {
    fn write(&self, to: &mut dyn BufMut, version: &Version) -> DifResult<()> {
        match self {
            PossiblyNullSurfaceIndex::Null(index) => (*index.inner() as u32) | 0x80000000u32,
            PossiblyNullSurfaceIndex::NonNull(index) => *index.inner() as u32,
        }
        .write(to, version)
    }
}
