use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Write;

use crate::bsp::build_bsp;
use crate::bsp::DIFBSPNode;
use cgmath::AbsDiffEq;
use cgmath::InnerSpace;
use cgmath::Transform;
use cgmath::Vector2;
use cgmath::Vector3;
use dif::interior::*;
use dif::types::*;
use image::codecs::png::PngEncoder;
use image::ImageBuffer;
use image::ImageEncoder;
use image::Rgb;
use itertools::Itertools;
use rectangle_pack::contains_smallest_box;
use rectangle_pack::pack_rects;
use rectangle_pack::volume_heuristic;
use rectangle_pack::GroupedRectsToPlace;
use rectangle_pack::RectToInsert;
use rectangle_pack::TargetBin;
use std::hash::Hash;

pub trait ProgressEventListener {
    fn progress(&mut self, current: u32, total: u32, status: String, finish_status: String);
}

#[derive(Clone)]
pub struct BSPReport {
    pub balance_factor: i32,
    pub hit: i32,
    pub total: usize,
    pub hit_area_percentage: f32,
}

#[derive(Clone)]
pub struct Triangle {
    pub verts: [Point3F; 3],
    pub plane: PlaneF,
    pub uv: [Point2F; 3],
    pub material: String,
    pub id: i32,
}

struct PolyGroup {
    min: Point3F,
    max: Point3F,
    polys: Vec<usize>,
}

impl PolyGroup {
    pub fn surface_area(&self) -> f32 {
        2.0 * ((self.max.x - self.min.x) * (self.max.y - self.min.y)
            + (self.max.y - self.min.y) * (self.max.z - self.min.z)
            + (self.max.z - self.min.z) * (self.max.x - self.min.x))
    }

    pub fn surface_area_if_add(&self, polygon: &Triangle) -> f32 {
        let new_min = Point3F::new(
            self.min
                .x
                .min(polygon.verts[0].x)
                .min(polygon.verts[1].x)
                .min(polygon.verts[2].x),
            self.min
                .y
                .min(polygon.verts[0].y)
                .min(polygon.verts[1].y)
                .min(polygon.verts[2].y),
            self.min
                .z
                .min(polygon.verts[0].z)
                .min(polygon.verts[1].z)
                .min(polygon.verts[2].z),
        );
        let new_max = Point3F::new(
            self.max
                .x
                .max(polygon.verts[0].x)
                .max(polygon.verts[1].x)
                .max(polygon.verts[2].x),
            self.max
                .y
                .max(polygon.verts[0].y)
                .max(polygon.verts[1].y)
                .max(polygon.verts[2].y),
            self.max
                .z
                .max(polygon.verts[0].z)
                .max(polygon.verts[1].z)
                .max(polygon.verts[2].z),
        );

        2.0 * ((new_max.x - new_min.x) * (new_max.y - new_min.y)
            + (new_max.y - new_min.y) * (new_max.z - new_min.z)
            + (new_max.z - new_min.z) * (new_max.x - new_min.x))
            - self.surface_area()
    }

    pub fn add_polygon(&mut self, polygon: &Triangle, poly_idx: usize) {
        self.min = Point3F::new(
            self.min
                .x
                .min(polygon.verts[0].x)
                .min(polygon.verts[1].x)
                .min(polygon.verts[2].x),
            self.min
                .y
                .min(polygon.verts[0].y)
                .min(polygon.verts[1].y)
                .min(polygon.verts[2].y),
            self.min
                .z
                .min(polygon.verts[0].z)
                .min(polygon.verts[1].z)
                .min(polygon.verts[2].z),
        );
        self.max = Point3F::new(
            self.max
                .x
                .max(polygon.verts[0].x)
                .max(polygon.verts[1].x)
                .max(polygon.verts[2].x),
            self.max
                .y
                .max(polygon.verts[0].y)
                .max(polygon.verts[1].y)
                .max(polygon.verts[2].y),
            self.max
                .z
                .max(polygon.verts[0].z)
                .max(polygon.verts[1].z)
                .max(polygon.verts[2].z),
        );
        self.polys.push(poly_idx);
    }
}

pub struct DIFBuilder {
    brushes: Vec<Triangle>,
    interior: Interior,
    face_to_surface: HashMap<i32, PossiblyNullSurfaceIndex>,
    face_to_plane: HashMap<i32, PlaneIndex>,
    plane_map: HashMap<OrdPlaneF, PlaneIndex>,
    point_map: HashMap<OrdPoint, PointIndex>,
    normal_map: HashMap<OrdPoint, NormalIndex>,
    texgen_map: HashMap<OrdTexGen, TexGenIndex>,
    emit_string_map: HashMap<Vec<u8>, EmitStringIndex>,
    mb_only: bool,
    bsp_report: BSPReport,
}

pub static mut POINT_EPSILON: f32 = 1e-6;
pub static mut PLANE_EPSILON: f32 = 1e-5;

impl DIFBuilder {
    pub fn new(mb_only: bool) -> DIFBuilder {
        return DIFBuilder {
            brushes: vec![],
            interior: empty_interior(),
            face_to_surface: HashMap::new(),
            face_to_plane: HashMap::new(),
            plane_map: HashMap::new(),
            point_map: HashMap::new(),
            normal_map: HashMap::new(),
            texgen_map: HashMap::new(),
            emit_string_map: HashMap::new(),
            mb_only: mb_only,
            bsp_report: BSPReport {
                balance_factor: 0,
                hit: 0,
                total: 0,
                hit_area_percentage: 0.0,
            },
        };
    }

    pub fn add_triangle(
        &mut self,
        v1: Point3F,
        v2: Point3F,
        v3: Point3F,
        uv1: Point2F,
        uv2: Point2F,
        uv3: Point2F,
        norm: Point3F,
        material: String,
    ) {
        let p = PlaneF {
            normal: norm,
            distance: -norm.dot(v1),
        };
        self.brushes.push(Triangle {
            verts: [v1, v2, v3],
            plane: p,
            uv: [uv1, uv2, uv3],
            material: material,
            id: self.brushes.len() as i32,
        });
    }

    pub fn build(
        mut self,
        progress_report_callback: &mut dyn ProgressEventListener,
    ) -> (Interior, BSPReport) {
        self.interior.bounding_box = get_bounding_box(&self.brushes);
        self.interior.bounding_box.min -= Point3F::new(3.0, 3.0, 3.0);
        self.interior.bounding_box.max += Point3F::new(3.0, 3.0, 3.0);
        self.interior.bounding_sphere = get_bounding_sphere(&self.brushes);
        self.export_brushes(progress_report_callback);
        self.interior.zones.push(Zone {
            portal_start: PortalIndex::new(0),
            portal_count: 0,
            surface_start: 0,
            surface_count: self.interior.surfaces.len() as _,
            static_mesh_start: StaticMeshIndex::new(0),
            static_mesh_count: 0,
            flags: 0,
        });
        self.export_coord_bins();
        if self.mb_only {
            self.interior
                .poly_list_plane_indices
                .push(PlaneIndex::from(0));
            self.interior
                .poly_list_point_indices
                .push(PointIndex::from(0));
            self.interior.poly_list_string_characters.push(0);
            self.interior.hull_plane_indices.push(PlaneIndex::from(0));
            self.interior
                .hull_emit_string_indices
                .push(EmitStringIndex::from(0));
            self.interior.convex_hull_emit_string_characters.push(0);
        } else {
            self.process_hull_poly_lists(); // Hull poly lists
        }
        // self.calculate_bsp_coverage();
        let balance_factor_save = self.bsp_report.balance_factor;
        self.bsp_report = self.interior.calculate_bsp_raycast_coverage();
        self.bsp_report.balance_factor = balance_factor_save;
        (self.interior, self.bsp_report)
    }

    fn export_brushes(&mut self, progress_report_callback: &mut dyn ProgressEventListener) {
        let grouped = self.group_polys();
        for i in 0..grouped.len() {
            progress_report_callback.progress(
                (i + 1) as u32,
                grouped.len() as u32,
                "Exporting \"convex\" hulls".to_string(),
                "Exported \"convex\" hulls".to_string(),
            );
            self.export_convex_hull(&grouped[i]);
        }
        // Ensure that ALL the polys are exported to a surface
        for poly in self.brushes.iter() {
            if !self.face_to_surface.contains_key(&poly.id) {
                println!("Face not exported???: {}", poly.id);
            }
        }
        let (bsp_root, plane_remap) = build_bsp(&self.brushes, progress_report_callback);
        self.bsp_report.balance_factor = bsp_root.balance_factor();
        self.export_bsp_node(&bsp_root, &plane_remap);
        // self.calculate_bsp_raycast_root_coverage(&bsp_root, &plane_remap);
    }

    fn export_bsp_node(&mut self, node: &DIFBSPNode, plane_remap: &Vec<PlaneF>) -> BSPIndex {
        if node.plane_index == None {
            if node.brush_list.len() > 0 {
                let surface_index = self.interior.solid_leaf_surfaces.len() as u32;
                let mut surface_count = 0;
                let mut exported = HashSet::new();
                node.brush_list.iter().for_each(|f| {
                    let surf_index = self.face_to_surface.get(&(f.id as i32)).unwrap();

                    let surf_index_num = match surf_index {
                        PossiblyNullSurfaceIndex::NonNull(idx) => *idx.inner() as u32,
                        PossiblyNullSurfaceIndex::Null(idx) => *idx.inner() as u32 | 0x80000000,
                    };

                    if !exported.contains(&surf_index_num) {
                        surface_count += 1;
                        exported.insert(surf_index_num);
                        self.interior.solid_leaf_surfaces.push(surf_index.clone());
                    }
                });
                if surface_count == 0 {
                    return BSPIndex {
                        leaf: true,
                        solid: false,
                        index: 0,
                    };
                } else {
                    let solid_leaf = BSPSolidLeaf {
                        surface_count: surface_count,
                        surface_index: surface_index.into(),
                    };
                    let leaf_index = self.interior.bsp_solid_leaves.len();
                    self.interior.bsp_solid_leaves.push(solid_leaf);
                    return BSPIndex {
                        leaf: true,
                        solid: true,
                        index: leaf_index as u32,
                    };
                }
            } else {
                let leaf_index = BSPIndex {
                    leaf: true,
                    solid: false,
                    index: 0,
                };
                return leaf_index;
            }
        } else {
            let node_index = self.interior.bsp_nodes.len();
            let bsp_node = BSPNode {
                front_index: BSPIndex {
                    index: 0,
                    leaf: true,
                    solid: false,
                },
                back_index: BSPIndex {
                    index: 0,
                    leaf: true,
                    solid: false,
                },
                plane_index: PlaneIndex::from(0),
            };

            self.interior.bsp_nodes.push(bsp_node);

            let node_plane = &plane_remap[node.plane_index.unwrap() as usize];
            let plane_index = self.export_plane(node_plane);
            let plane_flipped = *plane_index.inner() & 0x8000 != 0;

            let front_index = match node.front {
                Some(ref n) => self.export_bsp_node(n.as_ref(), plane_remap),
                None => BSPIndex {
                    leaf: true,
                    solid: false,
                    index: 0,
                },
            };
            let back_index = match node.back {
                Some(ref n) => self.export_bsp_node(n.as_ref(), plane_remap),
                None => BSPIndex {
                    leaf: true,
                    solid: false,
                    index: 0,
                },
            };
            self.interior.bsp_nodes[node_index].plane_index =
                PlaneIndex::from(*plane_index.inner() & 0x7FFF);
            if plane_flipped {
                self.interior.bsp_nodes[node_index].back_index = front_index;
                self.interior.bsp_nodes[node_index].front_index = back_index;
            } else {
                self.interior.bsp_nodes[node_index].back_index = back_index;
                self.interior.bsp_nodes[node_index].front_index = front_index;
            }

            return BSPIndex {
                leaf: false,
                solid: false,
                index: node_index as u32,
            };
        }
    }

    fn export_point(&mut self, point: &Point3F) -> PointIndex {
        let ord_point = OrdPoint::from(&point);
        if let Some(p) = self.point_map.get(&ord_point) {
            return *p;
        }
        let index = PointIndex::new(self.interior.points.len() as u32);
        self.interior.points.push(point.clone());
        self.interior.point_visibilities.push(0xff);
        self.point_map.insert(ord_point, index);
        return index;
    }

    fn export_tex_gen(&mut self, triangle: &Triangle) -> TexGenIndex {
        let index = TexGenIndex::new(self.interior.tex_gen_eqs.len() as _);

        let eq = get_tex_gen(triangle);

        let ord_texgen = OrdTexGen(TexGenEq {
            plane_x: eq.plane_x.clone(),
            plane_y: eq.plane_y.clone(),
        });
        if self.texgen_map.contains_key(&ord_texgen) {
            return *self.texgen_map.get(&ord_texgen).unwrap();
        }
        self.interior.tex_gen_eqs.push(eq);
        self.texgen_map.insert(ord_texgen, index);
        return index;
    }

    fn export_coord_bins(&mut self) {
        // There are always 256 of these (hard-coded in engine)
        for i in 0..256 {
            self.interior.coord_bins.push(CoordBin {
                bin_start: CoordBinIndex::new(i),
                bin_count: 1,
            });
        }
        // Split coordbins into 16x16 equal rect prisms in the xy plane
        // Probably a more efficient way to do this but this will work
        for i in 0..16 {
            let min_x = self.interior.bounding_box.min.x
                + (i as f32 * self.interior.bounding_box.extent().x / 16f32);
            let max_x = self.interior.bounding_box.min.x
                + ((i + 1) as f32 * self.interior.bounding_box.extent().x / 16f32);
            for j in 0..16 {
                let min_y = self.interior.bounding_box.min.y
                    + (j as f32 * self.interior.bounding_box.extent().y / 16f32);
                let max_y = self.interior.bounding_box.min.y
                    + ((j + 1) as f32 * self.interior.bounding_box.extent().y / 16f32);

                let bin_index = (i * 16) + j;
                let mut bin_count = 0;
                self.interior.coord_bins[bin_index as usize].bin_start =
                    CoordBinIndex::new(self.interior.coord_bin_indices.len() as _);
                for (k, hull) in self.interior.convex_hulls.iter().enumerate() {
                    if !(min_x > hull.max_x
                        || max_x < hull.min_x
                        || min_y > hull.max_y
                        || max_y < hull.min_y)
                    {
                        self.interior
                            .coord_bin_indices
                            .push(ConvexHullIndex::new(k as _));
                        bin_count += 1;
                    }
                }

                self.interior.coord_bins[bin_index as usize].bin_count = bin_count as _;
            }
        }
    }

    fn export_texture(&mut self, texture: String) -> TextureIndex {
        for i in 0..self.interior.material_names.len() {
            if self.interior.material_names[i] == texture {
                return TextureIndex::new(i as _);
            }
        }
        let index = TextureIndex::new(self.interior.material_names.len() as _);
        self.interior.material_names.push(texture);
        index
    }

    fn export_plane(&mut self, plane: &PlaneF) -> PlaneIndex {
        assert!(self.interior.planes.len() < 0x10000);
        let pord = OrdPlaneF::from(&plane);

        if self.plane_map.contains_key(&pord) {
            let pval = self.plane_map.get(&pord).unwrap();
            return *pval as PlaneIndex;
        }

        let mut pinvplane = plane.clone();
        pinvplane.normal *= -1.0;
        pinvplane.distance *= -1.0;

        let pord = OrdPlaneF::from(&pinvplane);

        if self.plane_map.contains_key(&pord) {
            let pval = self.plane_map.get(&pord).unwrap();
            let mut pindex = *pval.inner();
            pindex |= 0x8000;
            return PlaneIndex::from(pindex);
        }

        let index = PlaneIndex::new(self.interior.planes.len() as _);

        let normal_ord = OrdPoint::from(&plane.normal);

        let normal_map_idx = self.normal_map.get(&normal_ord);

        match normal_map_idx {
            Some(nidx) => {
                self.interior.planes.push(Plane {
                    normal_index: *nidx,
                    plane_distance: plane.distance,
                });
            }
            None => {
                let normal_index = NormalIndex::new(self.interior.normals.len() as _);
                self.normal_map.insert(normal_ord, normal_index);
                self.interior.normals.push(plane.normal);
                if !self.mb_only {
                    self.interior.normal2s.push(plane.normal);
                }

                self.interior.planes.push(Plane {
                    normal_index: normal_index as _,
                    plane_distance: plane.distance,
                });
            }
        }

        let pord = OrdPlaneF::from(&plane);

        self.plane_map.insert(pord, index);

        index
    }

    fn export_surface(&mut self, triangle: &Triangle) -> PossiblyNullSurfaceIndex {
        if self.face_to_surface.contains_key(&triangle.id) {
            return self.face_to_surface[&triangle.id].clone();
        }
        let index = SurfaceIndex::new(self.interior.surfaces.len() as _);

        self.face_to_surface
            .insert(triangle.id, PossiblyNullSurfaceIndex::NonNull(index));

        let plane_index = self.export_plane(&triangle.plane);
        let pflipped = plane_index.inner() & 0x8000 > 0;
        self.face_to_plane.insert(triangle.id, plane_index);

        let tex_gen_index = self.export_tex_gen(triangle);
        let winding_index = WindingIndexIndex::new(self.interior.indices.len() as _);
        let winding_length = 3;
        let p_idxs = triangle.verts.map(|p| self.export_point(&p));
        self.interior.indices.push(p_idxs[0]);
        self.interior.indices.push(p_idxs[1]);
        self.interior.indices.push(p_idxs[2]);

        let material_index = self.export_texture(triangle.material.clone());

        let mut fan_mask = 0b0;
        for i in 0..winding_length {
            fan_mask |= 1 << i;
        }

        let surface = Surface {
            winding_start: winding_index,
            winding_count: winding_length as _,
            plane_index: plane_index,
            plane_flipped: pflipped,
            texture_index: material_index,
            tex_gen_index: tex_gen_index,
            surface_flags: SurfaceFlags::OUTSIDE_VISIBLE,
            fan_mask: fan_mask as _,
            light_map: SurfaceLightMap {
                final_word: 0, // stEnc, lmapLogScaleX, lmapLogScaleY
                tex_gen_x_distance: 0.0,
                tex_gen_y_distance: 0.0,
            },
            light_count: 0,
            light_state_info_start: 0,
            map_offset_x: 0,
            map_offset_y: 0,
            map_size_x: 32,
            map_size_y: 32,
            brush_id: 0,
        };

        //TODO: Figure these out too
        self.interior
            .zone_surfaces
            .push(SurfaceIndex::new(self.interior.surfaces.len() as _));

        self.interior.normal_lmap_indices.push(LMapIndex::new(0u32));
        self.interior
            .alarm_lmap_indices
            .push(LMapIndex::new(0xffffffffu32));
        self.interior.surfaces.push(surface);

        PossiblyNullSurfaceIndex::NonNull(index)
    }

    fn export_null_surface(&mut self, triangle: &Triangle) -> PossiblyNullSurfaceIndex {
        if self.face_to_surface.contains_key(&triangle.id) {
            return self.face_to_surface[&triangle.id].clone();
        }
        let index = NullSurfaceIndex::new(self.interior.null_surfaces.len() as _);

        self.face_to_surface
            .insert(triangle.id, PossiblyNullSurfaceIndex::Null(index));

        let plane_index = self.export_plane(&triangle.plane);
        self.face_to_plane.insert(triangle.id, plane_index);

        let winding_index = WindingIndexIndex::new(self.interior.indices.len() as _);
        let winding_length = 3;
        let p_idxs = triangle.verts.map(|p| self.export_point(&p));
        self.interior.indices.push(p_idxs[0]);
        self.interior.indices.push(p_idxs[1]);
        self.interior.indices.push(p_idxs[2]);

        let null_surface = NullSurface {
            plane_index: plane_index,
            surface_flags: SurfaceFlags::OUTSIDE_VISIBLE,
            winding_start: winding_index,
            winding_count: winding_length as _,
        };

        self.interior.null_surfaces.push(null_surface);

        PossiblyNullSurfaceIndex::Null(index)
    }

    fn export_convex_hull(&mut self, poly_group: &Vec<usize>) -> usize {
        let b = poly_group
            .iter()
            .map(|i| self.brushes[*i].clone())
            .collect::<Vec<_>>();
        struct HullPoly {
            pub points: Vec<usize>,
            pub plane_index: usize,
        }
        #[derive(Hash, PartialEq, Eq)]
        struct EmitEdge {
            pub first: usize,
            pub last: usize,
        }

        let index = self.interior.convex_hulls.len();

        let hull_count: usize = poly_group.len() * 3; // b.vertices.vertex.len();
        assert!(hull_count < 0x10000);
        let bounding_box = BoxF::from_vertices(
            &b.iter()
                .flat_map(|t| t.verts.as_slice())
                .collect::<Vec<_>>(),
        );

        let hull = ConvexHull {
            hull_start: HullPointIndex::new(self.interior.hull_indices.len() as _),
            hull_count: hull_count as _,
            min_x: bounding_box.min.x,
            max_x: bounding_box.max.x,
            min_y: bounding_box.min.y,
            max_y: bounding_box.max.y,
            min_z: bounding_box.min.z,
            max_z: bounding_box.max.z,
            surface_start: HullSurfaceIndex::new(self.interior.hull_surface_indices.len() as _),
            surface_count: b.len() as _,
            plane_start: HullPlaneIndex::new(self.interior.hull_plane_indices.len() as _),
            poly_list_plane_start: PolyListPlaneIndex::new(
                self.interior.poly_list_plane_indices.len() as _,
            ),
            poly_list_point_start: PolyListPointIndex::new(
                self.interior.poly_list_point_indices.len() as _,
            ),
            poly_list_string_start: PolyListStringIndex::new(0),
            static_mesh: 0,
        };

        let mut hull_exported_points = vec![];
        let mut local_hull_point_map: HashMap<OrdPoint, usize> = HashMap::new();
        let mut local_hull_points = vec![];
        for t in b.iter() {
            for v in t.verts.iter() {
                hull_exported_points.push(self.export_point(v));
                let ord_point = OrdPoint::from(v);
                if !local_hull_point_map.contains_key(&ord_point) {
                    let index = local_hull_points.len();
                    local_hull_points.push(v.clone());
                    local_hull_point_map.insert(ord_point, index);
                }
            }
        }

        // Export hull points

        self.interior
            .hull_indices
            .append(&mut hull_exported_points.clone());
        if !self.mb_only {
            self.interior
                .poly_list_point_indices
                .append(&mut hull_exported_points.clone());
        }

        // Export hull planes
        let mut hull_plane_indices = b
            .iter()
            .map(|f| self.export_plane(&f.plane))
            .collect::<Vec<_>>();
        if !self.mb_only {
            self.interior
                .poly_list_plane_indices
                .append(&mut hull_plane_indices.clone());
            self.interior
                .hull_plane_indices
                .append(&mut hull_plane_indices);
        }

        // Export hull surfaces
        let mut hull_surface_indices = b
            .iter()
            .map(|f| {
                if f.material == "NULL" {
                    self.export_null_surface(f)
                } else {
                    self.export_surface(f)
                }
            })
            .collect::<Vec<_>>();
        self.interior
            .hull_surface_indices
            .append(&mut hull_surface_indices);

        // Hull polys
        let mut hull_polys = vec![];
        b.iter().for_each(|face| {
            hull_polys.push(HullPoly {
                points: face
                    .verts
                    .iter()
                    .map(|p| local_hull_point_map[&OrdPoint::from(p)])
                    .collect::<Vec<_>>(), // points.into_iter().map(|p| p).collect::<Vec<_>>(),
                plane_index: *self.face_to_plane[&face.id].inner() as usize,
            });
        });

        // Ok, now we have to construct an emit string for each vertex.  This should be fairly
        //  straightforward, the procedure is:
        // for each point:
        //   - find all polys that contain that point
        //   - find all points in those polys
        //   - find all edges  in those polys
        //   - enter the string
        //  The tricky bit is that we have to set up the emit indices to be relative to the
        //   hullindices.
        for (i, _) in hull_exported_points.into_iter().enumerate() {
            let mut emit_poly_indices = vec![];
            if !self.mb_only {
                // Collect emitted polys for this point
                for (j, poly) in hull_polys.iter().enumerate() {
                    if poly.points.contains(&i) {
                        emit_poly_indices.push(j);
                    }
                }
                // We also have to emit any polys that share the plane, but not necessarily the
                //  support point
                let mut new_indices = vec![];
                for (j, poly) in hull_polys.iter().enumerate() {
                    for &emit_poly in emit_poly_indices.iter() {
                        if emit_poly == j {
                            continue;
                        }

                        if hull_polys[emit_poly].plane_index == poly.plane_index {
                            if emit_poly_indices.contains(&j) {
                                continue;
                            }
                            new_indices.push(j);
                        }
                    }
                }
                emit_poly_indices.extend(new_indices);

                assert_ne!(emit_poly_indices.len(), 0);

                // Then generate all points and edges these polys contain
                let emit_points: Vec<usize> = Vec::from_iter(
                    emit_poly_indices
                        .iter()
                        .flat_map(|&poly| hull_polys[poly].points.clone())
                        .collect::<HashSet<_>>()
                        .into_iter(),
                );
                let emit_edges: Vec<EmitEdge> = Vec::from_iter(
                    emit_poly_indices
                        .iter()
                        .flat_map(|&poly| {
                            windows2_wrap(&hull_polys[poly].points).into_iter().map(
                                |(&first, &second)| EmitEdge {
                                    first: first.min(second),
                                    last: first.max(second),
                                },
                            )
                        })
                        .collect::<HashSet<_>>()
                        .into_iter(),
                );

                let mut emit_string: Vec<u8> = vec![];
                emit_string.push(emit_points.len() as _);
                for &point in &emit_points {
                    assert!(point < 0x100);
                    emit_string.push(point as _);
                }
                emit_string.push(emit_edges.len() as _);
                for edge in emit_edges {
                    assert!(edge.first < 0x100);
                    assert!(edge.last < 0x100);
                    emit_string.push(edge.first as _);
                    emit_string.push(edge.last as _);
                }
                emit_string.push(emit_poly_indices.len() as _);
                for poly_index in emit_poly_indices {
                    assert!(hull_polys[poly_index].points.len() < 0x100);
                    assert!(poly_index < 0x100);
                    emit_string.push(hull_polys[poly_index].points.len() as _);
                    emit_string.push(poly_index as _);
                    for point in hull_polys[poly_index].points.iter() {
                        if let Some(point_index) = emit_points.iter().position(|pt| pt == point) {
                            assert!(point_index < 0x100);
                            emit_string.push(point_index as _);
                        }
                    }
                }

                let emit_string_index = self.export_emit_string(emit_string);
                self.interior
                    .hull_emit_string_indices
                    .push(emit_string_index as _);
            }
        }

        self.interior.convex_hulls.push(hull);
        index
    }

    fn process_hull_poly_lists(&mut self) {
        self.interior.poly_list_plane_indices.clear();
        self.interior.poly_list_point_indices.clear();
        self.interior.poly_list_string_characters.clear();
        for hull in self.interior.convex_hulls.iter_mut() {
            let mut point_indices: Vec<u32> = vec![];
            let mut plane_indices: Vec<u16> = vec![];
            let mut temp_surfaces = vec![];

            // Extract all the surfaces from this hull into our temporary processing format
            for i in 0..hull.surface_count {
                let mut temp_surface = TempProcSurface::new();
                let surface_index = &self.interior.hull_surface_indices
                    [(i as u32 + hull.surface_start.inner()) as usize];
                {
                    match surface_index {
                        PossiblyNullSurfaceIndex::Null(idx) => {
                            let ns = &self.interior.null_surfaces[*idx.inner() as usize];
                            temp_surface.plane_index = *ns.plane_index.inner();
                            temp_surface.num_points = ns.winding_count as usize;
                            for j in 0..ns.winding_count {
                                temp_surface.point_indices[j as usize] = *self.interior.indices
                                    [*ns.winding_start.inner() as usize + j as usize]
                                    .inner();
                            }
                        }
                        PossiblyNullSurfaceIndex::NonNull(idx) => {
                            let s = &self.interior.surfaces[*idx.inner() as usize];
                            temp_surface.plane_index = *s.plane_index.inner();

                            let mut temp_indices = [0; 32];
                            let mut jdx = 1;
                            let mut j = 1;
                            while j < s.winding_count {
                                temp_indices[jdx] = j;
                                jdx += 1;
                                j += 2;
                            }
                            j = (s.winding_count - 1) & (!1);
                            while j > 0 {
                                temp_indices[jdx] = j;
                                j -= 2;
                            }
                            jdx = 0;
                            for j in 0..s.winding_count {
                                if s.fan_mask & (1 << j) > 0 {
                                    temp_surface.point_indices[jdx] =
                                        *self.interior.indices[*s.winding_start.inner() as usize
                                            + temp_indices[j as usize] as usize]
                                            .inner();
                                    jdx += 1;
                                }
                            }
                            temp_surface.num_points = jdx;
                        }
                    }
                }
                temp_surfaces.push(temp_surface);
            }

            // First order of business: extract all unique planes and points from
            //  the list of surfaces...
            for surf in temp_surfaces.iter() {
                let mut found = false;
                for plane_index in plane_indices.iter() {
                    if surf.plane_index == *plane_index {
                        found = true;
                        break;
                    }
                }
                if !found {
                    plane_indices.push(surf.plane_index);
                }
                for k in 0..surf.num_points {
                    found = false;
                    for point_index in point_indices.iter() {
                        if *point_index == surf.point_indices[k] {
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        point_indices.push(surf.point_indices[k]);
                    }
                }
            }

            // Now that we have all the unique points and planes, remap the surfaces in
            //  terms of the offsets into the unique point list...
            for surf in temp_surfaces.iter_mut() {
                for k in 0..surf.num_points {
                    let mut found = false;
                    for l in 0..point_indices.len() {
                        if point_indices[l] == surf.point_indices[k] {
                            surf.point_indices[k] = l as u32;
                            found = true;
                            break;
                        }
                    }
                    assert!(
                        found,
                        "Error remapping point indices in interior collision processing"
                    );
                }
            }

            // Ok, at this point, we have a list of unique points, unique planes, and the
            //  surfaces all remapped in those terms.  We need to check our error conditions
            //  that will make sure that we can properly encode this hull:
            assert!(
                plane_indices.len() < 256,
                "Error, > 256 planes on an interior hull"
            );
            assert!(
                point_indices.len() < 65536,
                "Error, > 65536 points on an interior hull"
            );
            assert!(
                temp_surfaces.len() < 256,
                "Error, > 256 surfaces on an interior hull"
            );

            // Now we group the planes together, and merge the closest groups until we're left
            //  with <= 8 groups
            let mut plane_groups = vec![];
            for plane_index in plane_indices.iter() {
                let mut pg = PlaneGrouping::new();
                pg.num_planes = 1;
                pg.plane_indices[0] = *plane_index;
                plane_groups.push(pg);
            }

            while plane_groups.len() > 8 {
                // Find the two closest groups.  If mdp(i, j) is the value of the
                //  largest pairwise dot product that can be computed from the vectors
                //  of group i, and group j, then the closest group pair is the one
                //  with the smallest value of mdp.
                let mut cur_min = 2.0;
                let mut first_group = -1;
                let mut second_group = -1;

                for j in 0..plane_groups.len() {
                    let first = &plane_groups[j];
                    for k in (j + 1)..plane_groups.len() {
                        let second = &plane_groups[k];
                        let mut max = -2.0;
                        for l in 0..first.num_planes {
                            for m in 0..second.num_planes {
                                let mut first_normal = self.interior.normals[*self.interior.planes
                                    [(first.plane_indices[l] & !0x8000) as usize]
                                    .normal_index
                                    .inner()
                                    as usize]
                                    .clone();
                                if first.plane_indices[l] & 0x8000 > 0 {
                                    first_normal *= -1.0;
                                }
                                let mut second_normal = self.interior.normals[*self.interior.planes
                                    [(second.plane_indices[m] & !0x8000) as usize]
                                    .normal_index
                                    .inner()
                                    as usize]
                                    .clone();
                                if second.plane_indices[m] & 0x8000 > 0 {
                                    second_normal *= -1.0;
                                }
                                let normal_dot = first_normal.dot(second_normal);
                                if normal_dot > max {
                                    max = normal_dot;
                                }
                            }
                        }

                        if max < cur_min {
                            cur_min = max;
                            first_group = j as i32;
                            second_group = k as i32;
                        }
                    }
                }
                assert!(
                    first_group != -1 && second_group != -1,
                    "Error, unable to find a suitable pairing?"
                );

                // Merge first and second
                let mut from = plane_groups[second_group as usize].clone();
                let to = &mut plane_groups[first_group as usize];
                while from.num_planes != 0 {
                    to.plane_indices[to.num_planes] = from.plane_indices[from.num_planes - 1];
                    to.num_planes += 1;
                    from.num_planes -= 1;
                }

                // And remove the merged group
                plane_groups.remove(second_group as usize);
            }

            // Assign a mask to each of the plane groupings
            for (j, plane_group) in plane_groups.iter_mut().enumerate() {
                plane_group.mask = (1 << j) as u8;
            }

            // Now, assign the mask to each of the temp polys
            for surf in temp_surfaces.iter_mut() {
                let mut assigned = false;
                for plane_group in plane_groups.iter() {
                    for l in 0..plane_group.num_planes {
                        if plane_group.plane_indices[l] == surf.plane_index {
                            surf.mask = plane_group.mask;
                            assigned = true;
                            break;
                        }
                    }
                    if assigned {
                        break;
                    }
                }
                assert!(
                    assigned,
                    "Error, missed a plane somewhere in the hull poly list!"
                );
            }

            // Copy the appropriate group mask to the plane masks
            let mut plane_masks = vec![];
            for plane_index in plane_indices.iter() {
                let mut found = false;
                for plane_group in plane_groups.iter() {
                    for l in 0..plane_group.num_planes {
                        if plane_group.plane_indices[l] == *plane_index {
                            plane_masks.push(plane_group.mask);
                            found = true;
                            break;
                        }
                    }
                    if found {
                        break;
                    }
                }
                if !found {
                    plane_masks.push(0);
                }
            }

            // And whip through the points, constructing the total mask for that point
            let mut point_masks = vec![];
            for (j, _) in point_indices.iter().enumerate() {
                point_masks.push(0);
                for surf in temp_surfaces.iter() {
                    for l in 0..surf.num_points {
                        if surf.point_indices[l] == j as u32 {
                            point_masks[j] |= surf.mask;
                            break;
                        }
                    }
                }
            }

            // Create the emit strings, and we're done!

            // Set the range of planes
            hull.poly_list_plane_start =
                PolyListPlaneIndex::from(self.interior.poly_list_plane_indices.len() as u32);

            for plane_index in plane_indices.iter() {
                self.interior
                    .poly_list_plane_indices
                    .push(PlaneIndex::from(*plane_index));
            }

            // Set the range of points
            hull.poly_list_point_start =
                PolyListPointIndex::from(self.interior.poly_list_point_indices.len() as u32);
            for point_index in point_indices.iter() {
                self.interior
                    .poly_list_point_indices
                    .push(PointIndex::from(*point_index));
            }

            // Now the emit string.  The emit string goes like: (all fields are bytes)
            //  NumPlanes (PLMask) * NumPlanes
            //  NumPointsHi NumPointsLo (PtMask) * NumPoints
            //  NumSurfaces
            //   (NumPoints SurfaceMask PlOffset (PtOffsetHi PtOffsetLo) * NumPoints) * NumSurfaces
            //
            let mut _string_len = 1 + plane_indices.len() + 2 + point_indices.len() + 1;
            for surf in temp_surfaces.iter() {
                _string_len += 1 + 1 + 1 + (surf.num_points * 2);
            }

            hull.poly_list_string_start =
                PolyListStringIndex::from(self.interior.poly_list_string_characters.len() as u32);

            // Planes
            self.interior
                .poly_list_string_characters
                .push(plane_indices.len() as u8);
            for plane_index in plane_masks.iter() {
                self.interior.poly_list_string_characters.push(*plane_index);
            }

            // Points
            self.interior
                .poly_list_string_characters
                .push(((point_indices.len() >> 8) & 0xFF) as u8);
            self.interior
                .poly_list_string_characters
                .push((point_indices.len() & 0xFF) as u8);
            for point_index in point_masks.iter() {
                self.interior.poly_list_string_characters.push(*point_index);
            }

            // Surfaces
            self.interior
                .poly_list_string_characters
                .push(temp_surfaces.len() as u8);
            for surf in temp_surfaces.iter() {
                self.interior
                    .poly_list_string_characters
                    .push(surf.num_points as u8);
                self.interior
                    .poly_list_string_characters
                    .push(surf.mask as u8);

                let mut found = false;
                for (k, plane_index) in plane_indices.iter().enumerate() {
                    if *plane_index == surf.plane_index {
                        self.interior.poly_list_string_characters.push(k as u8);
                        found = true;
                        break;
                    }
                }
                assert!(found, "Error, missed a plane in the poly list!");
                for k in 0..surf.num_points {
                    self.interior
                        .poly_list_string_characters
                        .push(((surf.point_indices[k] >> 8) & 0xFF) as u8);
                    self.interior
                        .poly_list_string_characters
                        .push((surf.point_indices[k] & 0xFF) as u8);
                }
            }
        }
    }

    fn export_emit_string(&mut self, string: Vec<u8>) -> EmitStringIndex {
        let index =
            EmitStringIndex::new(self.interior.convex_hull_emit_string_characters.len() as _);
        if self.emit_string_map.contains_key(&string) {
            return *self.emit_string_map.get(&string).unwrap();
        }
        self.emit_string_map.insert(string.clone(), index);
        self.interior
            .convex_hull_emit_string_characters
            .extend(string);
        index
    }

    fn _calculate_bsp_coverage(&self) {
        let root = &self.interior.bsp_nodes[0];
        let mut used_surfaces = HashSet::new();
        self._calculate_bsp_coverage_rec(root, &mut used_surfaces);
        println!(
            "BSP Coverage: {} / {} surfaces ({}%)",
            used_surfaces.len(),
            self.interior.surfaces.len(),
            (used_surfaces.len() as f32 / self.interior.surfaces.len() as f32) * 100.0
        );
    }

    fn _calculate_bsp_coverage_rec(&self, bsp_node: &BSPNode, used_surfaces: &mut HashSet<u16>) {
        if bsp_node.front_index.solid && bsp_node.front_index.leaf {
            let leaf = &self.interior.bsp_solid_leaves[bsp_node.front_index.index as usize];
            let surfaces = &self.interior.solid_leaf_surfaces[(*leaf.surface_index.inner() as usize)
                ..((*leaf.surface_index.inner() + leaf.surface_count as u32) as usize)];
            surfaces.iter().for_each(|s| match s {
                PossiblyNullSurfaceIndex::NonNull(s_inner) => {
                    used_surfaces.insert(*s_inner.inner());
                }
                _ => {}
            });
        } else if !bsp_node.front_index.leaf {
            self._calculate_bsp_coverage_rec(
                &self.interior.bsp_nodes[bsp_node.front_index.index as usize],
                used_surfaces,
            );
        }
        if bsp_node.back_index.solid && bsp_node.back_index.leaf {
            let leaf = &self.interior.bsp_solid_leaves[bsp_node.back_index.index as usize];
            let surfaces = &self.interior.solid_leaf_surfaces[(*leaf.surface_index.inner() as usize)
                ..((*leaf.surface_index.inner() + leaf.surface_count as u32) as usize)];
            surfaces.iter().for_each(|s| match s {
                PossiblyNullSurfaceIndex::NonNull(s_inner) => {
                    used_surfaces.insert(*s_inner.inner());
                }
                _ => {}
            });
        } else if !bsp_node.back_index.leaf {
            self._calculate_bsp_coverage_rec(
                &self.interior.bsp_nodes[bsp_node.back_index.index as usize],
                used_surfaces,
            );
        }
    }

    fn _calculate_bsp_raycast_root_coverage(
        &self,
        bsp_root: &DIFBSPNode,
        bsp_plane_list: &[PlaneF],
    ) {
        let mut hit = 0;
        self.interior
            .surfaces
            .iter()
            .enumerate()
            .for_each(|(_, s)| {
                let points = &self.interior.indices[(*s.winding_start.inner() as usize)
                    ..((*s.winding_start.inner() + s.winding_count) as usize)]
                    .iter()
                    .map(|i| self.interior.points[*i.inner() as usize])
                    .collect::<Vec<_>>();
                let mut avg_point: Point3F = points.iter().sum();
                avg_point /= s.winding_count as f32;

                let plane_index = *s.plane_index.inner() & 0x7FFF;
                let norm = self.interior.normals[*self.interior.planes[plane_index as usize]
                    .normal_index
                    .inner() as usize];

                let start = avg_point
                    + (norm
                        * match s.plane_flipped {
                            true => -1.0,
                            false => 1.0,
                        })
                        * 0.1;
                let end = avg_point
                    - (norm
                        * match s.plane_flipped {
                            true => -1.0,
                            false => 1.0,
                        })
                        * 0.1;
                let pidx = usize::MAX;

                if bsp_root.ray_cast(start, end, pidx, bsp_plane_list) {
                    hit += 1;
                } else {
                    // println!("Miss: surface {} plane {}", i, plane_index);
                    // bsp_root.ray_cast(start, end, pidx, bsp_plane_list);
                }
            });
        println!(
            "BSP Raycast Coverage: {} / {} surfaces ({})",
            hit,
            self.interior.surfaces.len(),
            (hit as f32 / self.interior.surfaces.len() as f32) * 100.0
        );
    }

    fn subdivide_polys_into_groups(&self, polys: &Vec<usize>) -> Vec<Vec<usize>> {
        let mut groups: Vec<PolyGroup> = vec![];
        for poly_idx in polys.iter() {
            let poly = &self.brushes[*poly_idx];
            if groups.len() > 0 {
                let poly_min = Point3F::new(
                    poly.verts[0].x.min(poly.verts[1].x).min(poly.verts[2].x),
                    poly.verts[0].y.min(poly.verts[1].y).min(poly.verts[2].y),
                    poly.verts[0].z.min(poly.verts[1].z).min(poly.verts[2].z),
                );
                let poly_max = Point3F::new(
                    poly.verts[0].x.max(poly.verts[1].x).max(poly.verts[2].x),
                    poly.verts[0].y.max(poly.verts[1].y).max(poly.verts[2].y),
                    poly.verts[0].z.max(poly.verts[1].z).max(poly.verts[2].z),
                );
                let new_cost = 2.0
                    * ((poly_max.x - poly_min.x) * (poly_max.y - poly_min.y)
                        + (poly_max.y - poly_min.y) * (poly_max.z - poly_min.z)
                        + (poly_max.z - poly_min.z) * (poly_max.x - poly_min.x));

                let mut costs = groups
                    .iter()
                    .map(|g| g.surface_area_if_add(poly))
                    .collect::<Vec<f32>>();
                costs.push(new_cost);
                // Get minimum cost
                let min_pos = costs
                    .iter()
                    .enumerate()
                    .max_by(|(_, a), (_, b)| a.total_cmp(b))
                    .map(|(index, _)| index)
                    .unwrap();
                if min_pos == costs.len() - 1 {
                    let g = PolyGroup {
                        min: poly_min,
                        max: poly_max,
                        polys: vec![*poly_idx],
                    };
                    groups.push(g);
                } else {
                    let g = groups.get_mut(min_pos).unwrap();
                    g.add_polygon(poly, *poly_idx);
                }
            } else {
                let g = PolyGroup {
                    min: Point3F::new(
                        poly.verts[0].x.min(poly.verts[1].x).min(poly.verts[2].x),
                        poly.verts[0].y.min(poly.verts[1].y).min(poly.verts[2].y),
                        poly.verts[0].z.min(poly.verts[1].z).min(poly.verts[2].z),
                    ),
                    max: Point3F::new(
                        poly.verts[0].x.max(poly.verts[1].x).max(poly.verts[2].x),
                        poly.verts[0].y.max(poly.verts[1].y).max(poly.verts[2].y),
                        poly.verts[0].z.max(poly.verts[1].z).max(poly.verts[2].z),
                    ),
                    polys: vec![*poly_idx],
                };
                groups.push(g);
            }
        }
        groups.iter().map(|g| g.polys.clone()).collect::<Vec<_>>()
    }

    fn group_polys(&self) -> Vec<Vec<usize>> {
        let mut bounding_box = get_bounding_box(&self.brushes);
        bounding_box.min -= Point3F::new(3.0, 3.0, 3.0);
        bounding_box.max += Point3F::new(3.0, 3.0, 3.0);

        let mut poly_bins: [Vec<usize>; 256] = std::array::from_fn(|_| Vec::new());

        for i in 0..16 {
            let min_x = bounding_box.min.x + (i as f32 * bounding_box.extent().x / 16f32);
            let max_x = bounding_box.min.x + ((i + 1) as f32 * bounding_box.extent().x / 16f32);
            for j in 0..16 {
                let min_y = bounding_box.min.y + (j as f32 * bounding_box.extent().y / 16f32);
                let max_y = bounding_box.min.y + ((j + 1) as f32 * bounding_box.extent().y / 16f32);

                let bin_index = (i * 16) + j;

                for (k, poly) in self.brushes.iter().enumerate() {
                    let poly_min = Point3F::new(
                        poly.verts[0].x.min(poly.verts[1].x).min(poly.verts[2].x),
                        poly.verts[0].y.min(poly.verts[1].y).min(poly.verts[2].y),
                        poly.verts[0].z.min(poly.verts[1].z).min(poly.verts[2].z),
                    );
                    let poly_max = Point3F::new(
                        poly.verts[0].x.max(poly.verts[1].x).max(poly.verts[2].x),
                        poly.verts[0].y.max(poly.verts[1].y).max(poly.verts[2].y),
                        poly.verts[0].z.max(poly.verts[1].z).max(poly.verts[2].z),
                    );

                    if !(min_x > poly_max.x
                        || max_x < poly_min.x
                        || min_y > poly_max.y
                        || max_y < poly_min.y)
                    {
                        poly_bins[bin_index].push(k);
                    }
                }
            }
        }

        // let mut contain_set = HashSet::new();
        // // Ensure ALL polys are present in the bins
        // for i in 0..256 {
        //     for poly in poly_bins[i].iter() {
        //         contain_set.insert(self.brushes[*poly].id);
        //     }
        // }
        // // Check
        // for poly in self.brushes.iter() {
        //     if !contain_set.contains(&poly.id) {
        //         println!("Poly {} not in any bin!", poly.id);

        //         let poly_min = Point3F::new(
        //             poly.verts[0].x.min(poly.verts[1].x).min(poly.verts[2].x),
        //             poly.verts[0].y.min(poly.verts[1].y).min(poly.verts[2].y),
        //             poly.verts[0].z.min(poly.verts[1].z).min(poly.verts[2].z),
        //         );
        //         let poly_max = Point3F::new(
        //             poly.verts[0].x.max(poly.verts[1].x).max(poly.verts[2].x),
        //             poly.verts[0].y.max(poly.verts[1].y).max(poly.verts[2].y),
        //             poly.verts[0].z.max(poly.verts[1].z).max(poly.verts[2].z),
        //         );

        //         println!("Min: {} {} {}", poly_min.x, poly_min.y, poly_min.z);
        //         println!("Max: {} {} {}", poly_max.x, poly_max.y, poly_max.z);
        //         println!("Container");
        //         println!(
        //             "Min: {} {} {}",
        //             bounding_box.min.x, bounding_box.min.y, bounding_box.min.z
        //         );
        //         println!(
        //             "Max: {} {} {}",
        //             bounding_box.max.x, bounding_box.max.y, bounding_box.max.z
        //         );
        //     }
        // }

        let mut ret: Vec<Vec<usize>> = vec![];
        for i in 0..256 {
            let subdivided = self.subdivide_polys_into_groups(&poly_bins[i]);
            for group in subdivided {
                ret.push(group);
            }
        }

        let mut used_polys = HashSet::new();
        let mut final_list: Vec<_> = vec![];
        for poly_list in ret.iter() {
            let mut new_list = vec![];
            for g in poly_list.iter() {
                if !used_polys.contains(g) {
                    new_list.push(*g);
                    used_polys.insert(*g);
                }
            }
            if new_list.len() > 0 {
                final_list.push(new_list);
            }
        }
        final_list
    }
}

pub fn windows2_wrap<T>(input: &Vec<T>) -> Vec<(&T, &T)>
where
    T: Copy,
{
    let mut results = vec![];
    for i in 0..input.len() {
        results.push((&input[i], &input[(i + 1) % input.len()]));
    }
    results
}

pub fn get_bounding_box(brushes: &[Triangle]) -> BoxF {
    BoxF::from_vertices(&brushes.iter().flat_map(|t| &t.verts).collect::<Vec<_>>())
}

pub fn get_bounding_box_not_owned(brushes: &[&Triangle]) -> BoxF {
    BoxF::from_vertices(&brushes.iter().flat_map(|t| &t.verts).collect::<Vec<_>>())
}

fn get_bounding_sphere(brushes: &[Triangle]) -> SphereF {
    let b = get_bounding_box(brushes);

    SphereF {
        origin: b.center(),
        radius: (b.max - b.center()).magnitude(),
    }
}

fn empty_interior() -> Interior {
    Interior {
        detail_level: 0,
        min_pixels: 250,
        bounding_box: BoxF {
            min: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            max: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        },
        bounding_sphere: SphereF {
            origin: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            radius: 0.0,
        },
        has_alarm_state: 0,
        num_light_state_entries: 0,
        normals: vec![],
        planes: vec![],
        points: vec![],
        point_visibilities: vec![],
        tex_gen_eqs: vec![],
        bsp_nodes: vec![],
        bsp_solid_leaves: vec![],
        material_names: vec![],
        indices: vec![],
        winding_indices: vec![],
        edges: vec![],
        zones: vec![],
        zone_surfaces: vec![],
        zone_static_meshes: vec![],
        zone_portal_lists: vec![],
        portals: vec![],
        surfaces: vec![],
        edge2s: vec![],
        normal2s: vec![],
        normal_indices: vec![],
        normal_lmap_indices: vec![],
        alarm_lmap_indices: vec![],
        null_surfaces: vec![],
        light_maps: vec![],
        solid_leaf_surfaces: vec![],
        animated_lights: vec![],
        light_states: vec![],
        state_datas: vec![],
        state_data_buffers: vec![],
        flags: 0,
        name_buffer_characters: vec![],
        sub_objects: vec![],
        convex_hulls: vec![],
        convex_hull_emit_string_characters: vec![],
        hull_indices: vec![],
        hull_plane_indices: vec![],
        hull_emit_string_indices: vec![],
        hull_surface_indices: vec![],
        poly_list_plane_indices: vec![],
        poly_list_point_indices: vec![],
        poly_list_string_characters: vec![],
        coord_bins: vec![],
        coord_bin_indices: vec![],
        coord_bin_mode: 0,
        base_ambient_color: ColorI {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        },
        alarm_ambient_color: ColorI {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        },
        static_meshes: vec![],
        tex_normals: vec![],
        tex_matrices: vec![],
        tex_matrix_indices: vec![],
        extended_light_map_data: 0,
        light_map_border_size: 0,
    }
}

fn empty_lightmap(r: u8, g: u8, b: u8) -> PNG {
    let mut img = ImageBuffer::new(256, 256);
    for (_, _, pixel) in img.enumerate_pixels_mut() {
        *pixel = image::Rgb([r, g, b]);
    }
    let mut v = Vec::new();
    let png = PngEncoder::new(v.by_ref());
    let _ = png
        .write_image(&img, 256, 256, image::ExtendedColorType::Rgb8)
        .unwrap();

    PNG { data: v }
}

fn _filled_lightmap(data: &[u8]) -> PNG {
    let mut img = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(256, 256);
    img.copy_from_slice(data);
    let mut v = Vec::new();
    let png = PngEncoder::new(v.by_ref());
    let _ = png
        .write_image(&img, 256, 256, image::ExtendedColorType::Rgb8)
        .unwrap();

    PNG { data: v }
}

struct TempProcSurface {
    pub num_points: usize,
    pub point_indices: [u32; 32],
    pub plane_index: u16,
    pub mask: u8,
}

impl TempProcSurface {
    pub fn new() -> Self {
        TempProcSurface {
            num_points: 0,
            point_indices: [0; 32],
            plane_index: 0,
            mask: 0,
        }
    }
}

#[derive(Copy, Clone)]
struct PlaneGrouping {
    pub num_planes: usize,
    pub plane_indices: [u16; 32],
    pub mask: u8,
}

impl PlaneGrouping {
    pub fn new() -> Self {
        PlaneGrouping {
            num_planes: 0,
            plane_indices: [0; 32],
            mask: 0,
        }
    }
}

#[derive(Clone, PartialOrd)]
pub struct OrdPoint {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl OrdPoint {
    pub fn from(p: &Point3F) -> Self {
        OrdPoint {
            x: p.x,
            y: p.y,
            z: p.z,
        }
    }
}

impl PartialEq for OrdPoint {
    fn eq(&self, other: &Self) -> bool {
        self.x.abs_diff_eq(&other.x, unsafe { POINT_EPSILON })
            && self.y.abs_diff_eq(&other.y, unsafe { POINT_EPSILON })
            && self.z.abs_diff_eq(&other.z, unsafe { POINT_EPSILON })
    }
}

impl Eq for OrdPoint {}

impl Hash for OrdPoint {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let x = (self.x.floor() as u32 >> 5) & 0xf;
        let y = (self.y.floor() as u32 >> 5) & 0xf;
        let z = (self.z.floor() as u32 >> 5) & 0xf;

        let hash_val = (x << 8) | (y << 4) | z;
        hash_val.hash(state);
    }
}

#[derive(Clone, PartialOrd)]
pub struct OrdPlaneF {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub d: f32,
}

impl OrdPlaneF {
    pub fn from(v: &PlaneF) -> Self {
        OrdPlaneF {
            x: v.normal.x,
            y: v.normal.y,
            z: v.normal.z,
            d: v.distance,
        }
    }
}

impl PartialEq for OrdPlaneF {
    fn eq(&self, other: &Self) -> bool {
        self.x * other.x + self.y * other.y + self.z * other.z > 0.999
            && self.d.abs_diff_eq(&other.d, unsafe { PLANE_EPSILON })
    }
}

impl Eq for OrdPlaneF {}

impl Hash for OrdPlaneF {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut mul = self.x.abs().max(self.y.abs()).max(self.z.abs());
        mul = (mul * 100.0 + 0.5).floor() / 100.0;
        let val = mul * ((self.d.abs() * 100.0 + 0.5).floor() / 100.0);
        let hash_val = (val as u32) % (1 << 12);
        hash_val.hash(state);
    }
}

struct OrdTexGen(TexGenEq);

impl PartialEq for OrdTexGen {
    fn eq(&self, other: &Self) -> bool {
        self.0
            .plane_x
            .normal
            .x
            .abs_diff_eq(&other.0.plane_x.normal.x, 1e-5)
            && self
                .0
                .plane_x
                .normal
                .y
                .abs_diff_eq(&other.0.plane_x.normal.y, 1e-5)
            && self
                .0
                .plane_x
                .normal
                .z
                .abs_diff_eq(&other.0.plane_x.normal.z, 1e-5)
            && self
                .0
                .plane_x
                .distance
                .abs_diff_eq(&other.0.plane_x.distance, 1e-5)
            && self
                .0
                .plane_y
                .normal
                .x
                .abs_diff_eq(&other.0.plane_y.normal.x, 1e-5)
            && self
                .0
                .plane_y
                .normal
                .y
                .abs_diff_eq(&other.0.plane_y.normal.y, 1e-5)
            && self
                .0
                .plane_y
                .normal
                .z
                .abs_diff_eq(&other.0.plane_y.normal.z, 1e-5)
            && self
                .0
                .plane_y
                .distance
                .abs_diff_eq(&other.0.plane_y.distance, 1e-5)
    }
}

impl Eq for OrdTexGen {}

impl Hash for OrdTexGen {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut mul = self
            .0
            .plane_x
            .normal
            .x
            .abs()
            .max(self.0.plane_x.normal.y.abs())
            .max(self.0.plane_x.normal.z.abs());
        mul = (mul * 100.0 + 0.5).floor() / 100.0;
        let val = mul * ((self.0.plane_x.distance.abs() * 100.0 + 0.5).floor() / 100.0);
        let hash_val = (val as u32) % (1 << 12);
        hash_val.hash(state);

        // Same for plane y
        let mut mul = self
            .0
            .plane_y
            .normal
            .x
            .abs()
            .max(self.0.plane_y.normal.y.abs())
            .max(self.0.plane_y.normal.z.abs());

        mul = (mul * 100.0 + 0.5).floor() / 100.0;
        let val = mul * ((self.0.plane_y.distance.abs() * 100.0 + 0.5).floor() / 100.0);
        let hash_val = (val as u32) % (1 << 12);
        hash_val.hash(state);
    }
}

pub trait RaycastCalc {
    fn bsp_ray_cast(
        &self,
        node: &BSPIndex,
        plane_index: &u16,
        start: Point3F,
        end: Point3F,
    ) -> bool;

    fn calculate_bsp_raycast_coverage(&mut self) -> BSPReport;
}

impl RaycastCalc for Interior {
    fn calculate_bsp_raycast_coverage(&mut self) -> BSPReport {
        let mut hit = 0;
        let mut total_surface_area = 0.0;
        let mut hit_surface_area = 0.0;
        self.surfaces.iter().enumerate().for_each(|(_, s)| {
            let points = &self.indices[(*s.winding_start.inner() as usize)
                ..((*s.winding_start.inner() + s.winding_count) as usize)]
                .iter()
                .map(|i| self.points[*i.inner() as usize])
                .collect::<Vec<_>>();
            let mut avg_point: Point3F = points.iter().sum();
            avg_point /= s.winding_count as f32;

            let mut surface_area = 0.0;
            for i in 0..points.len() {
                surface_area += (points[i] - avg_point)
                    .cross(points[(i + 1) % points.len()] - avg_point)
                    .magnitude()
                    / 2.0;
            }
            total_surface_area += surface_area;

            let plane_index = *s.plane_index.inner() & 0x7FFF;
            let norm =
                self.normals[*self.planes[plane_index as usize].normal_index.inner() as usize];

            let start = avg_point
                + (norm
                    * match s.plane_flipped {
                        true => -1.0,
                        false => 1.0,
                    })
                    * 0.1;
            let end = avg_point
                - (norm
                    * match s.plane_flipped {
                        true => -1.0,
                        false => 1.0,
                    })
                    * 0.1;
            let pidx = u16::MAX;
            let start_node_index = BSPIndex {
                index: 0,
                leaf: false,
                solid: false,
            };

            if self.bsp_ray_cast(&start_node_index, &pidx, start, end) {
                hit += 1;
                hit_surface_area += surface_area;
            } else {
                // println!("Miss: surface {} plane {}", i, plane_index);
                // self.bsp_ray_cast(&start_node_index, &pidx, start, end);
            }
        });
        BSPReport {
            hit,
            balance_factor: 0,
            total: self.surfaces.len(),
            hit_area_percentage: (hit_surface_area / total_surface_area) * 100.0,
        }
    }

    fn bsp_ray_cast(
        &self,
        node: &BSPIndex,
        plane_index: &u16,
        start: Point3F,
        end: Point3F,
    ) -> bool {
        if !node.leaf {
            use std::cmp::Ordering;
            let node_value = &self.bsp_nodes[node.index as usize];
            let node_plane_index = *node_value.plane_index.inner();
            let plane_flipped = node_plane_index & 0x8000 > 0;
            let plane_value = &self.planes[(node_plane_index & 0x7FFF) as usize];
            let mut plane_norm = self.normals[*plane_value.normal_index.inner() as usize];
            if plane_flipped {
                plane_norm = -plane_norm;
            }
            let mut plane_d = plane_value.plane_distance;
            if plane_flipped {
                plane_d = -plane_d;
            }

            let s_side_value = plane_norm.dot(start) + plane_d;
            let e_side_value = plane_norm.dot(end) + plane_d;
            let s_side = s_side_value.total_cmp(&0.0);
            let e_side = e_side_value.total_cmp(&0.0);

            match (s_side, e_side) {
                (Ordering::Greater, Ordering::Greater)
                | (Ordering::Greater, Ordering::Equal)
                | (Ordering::Equal, Ordering::Greater) => {
                    self.bsp_ray_cast(&node_value.front_index, &plane_index, start, end)
                }
                (Ordering::Greater, Ordering::Less) => {
                    let intersect_t =
                        (-plane_d - start.dot(plane_norm)) / (end - start).dot(plane_norm);
                    let ip = start + (end - start) * intersect_t;
                    if self.bsp_ray_cast(&node_value.front_index, &plane_index, start, ip) {
                        return true;
                    }
                    self.bsp_ray_cast(
                        &node_value.back_index,
                        node_value.plane_index.inner(),
                        ip,
                        end,
                    )
                }
                (Ordering::Less, Ordering::Greater) => {
                    let intersect_t =
                        (-plane_d - start.dot(plane_norm)) / (end - start).dot(plane_norm);
                    let ip = start + (end - start) * intersect_t;
                    if self.bsp_ray_cast(&node_value.back_index, &plane_index, start, ip) {
                        return true;
                    }
                    self.bsp_ray_cast(
                        &node_value.front_index,
                        node_value.plane_index.inner(),
                        ip,
                        end,
                    )
                }
                (Ordering::Less, Ordering::Less)
                | (Ordering::Less, Ordering::Equal)
                | (Ordering::Equal, Ordering::Less) => {
                    self.bsp_ray_cast(&node_value.back_index, &plane_index, start, end)
                }
                (Ordering::Equal, Ordering::Equal) => {
                    if self.bsp_ray_cast(&node_value.front_index, &plane_index, start, end) {
                        return true;
                    }
                    if self.bsp_ray_cast(&node_value.back_index, &plane_index, start, end) {
                        return true;
                    }
                    false
                }
            }
        } else if node.solid {
            let leaf = &self.bsp_solid_leaves[node.index as usize];
            let surfaces = &self.solid_leaf_surfaces[(*leaf.surface_index.inner() as usize)
                ..((*leaf.surface_index.inner() + leaf.surface_count as u32) as usize)];
            let mut found = 0;
            surfaces.iter().for_each(|s| {
                match s {
                    PossiblyNullSurfaceIndex::NonNull(s_index) => {
                        let surf = &self.surfaces[*s_index.inner() as usize];
                        let surf_plane_index = *surf.plane_index.inner();
                        if surf_plane_index & 0x7FFF == *plane_index & 0x7FFF {
                            found += 1;
                        }
                    }
                    _ => {}
                };
            });
            if found == 0 {
                return false;
            }
            return true;
        } else {
            return false;
        }
    }
}

fn get_tex_gen(tri: &Triangle) -> TexGenEq {
    gen_tex_gen_from_points(
        tri.verts[0],
        tri.verts[1],
        tri.verts[2],
        tri.uv[0],
        tri.uv[1],
        tri.uv[2],
    )
}

fn gen_tex_gen_from_points(
    point0: Vector3<f32>,
    point1: Vector3<f32>,
    point2: Vector3<f32>,
    uv0: Vector2<f32>,
    uv1: Vector2<f32>,
    uv2: Vector2<f32>,
) -> TexGenEq {
    let tg = TexGenEq {
        plane_x: solve_matrix(point0, point1, point2, uv0.x, uv1.x, uv2.x),
        plane_y: solve_matrix(point0, point1, point2, uv0.y, uv1.y, uv2.y),
    };

    fn eps_fract(a: f32, b: f32) -> bool {
        let mut afract = a.fract();
        let mut bfract = b.fract();

        if afract < 0f32 {
            afract += 1f32;
        }
        if bfract < 0f32 {
            bfract += 1f32;
        }

        if (afract - bfract).abs() < 0.01 || (afract - bfract).abs() > 0.99 {
            true
        } else {
            println!(
                "{} {} {} {} => {}",
                a,
                b,
                afract,
                bfract,
                (afract - bfract).abs()
            );
            false
        }
    }

    //    assert!(eps_fract(tg.plane_x.normal.x * point0.x + tg.plane_x.normal.y * point0.y + tg.plane_x.normal.z * point0.z, uv0.x));
    //    assert!(eps_fract(tg.plane_x.normal.x * point1.x + tg.plane_x.normal.y * point1.y + tg.plane_x.normal.z * point1.z, uv1.x));
    //    assert!(eps_fract(tg.plane_x.normal.x * point2.x + tg.plane_x.normal.y * point2.y + tg.plane_x.normal.z * point2.z, uv2.x));
    //
    //    assert!(eps_fract(tg.plane_y.normal.x * point0.x + tg.plane_y.normal.y * point0.y + tg.plane_y.normal.z * point0.z, uv0.y));
    //    assert!(eps_fract(tg.plane_y.normal.x * point1.x + tg.plane_y.normal.y * point1.y + tg.plane_y.normal.z * point1.z, uv1.y));
    //    assert!(eps_fract(tg.plane_y.normal.x * point2.x + tg.plane_y.normal.y * point2.y + tg.plane_y.normal.z * point2.z, uv2.y));

    tg
}

fn solve_matrix(
    point0: Vector3<f32>,
    point1: Vector3<f32>,
    point2: Vector3<f32>,
    uv0: f32,
    uv1: f32,
    uv2: f32,
) -> PlaneF {
    use nalgebra::base::DMatrix;
    use nalgebra::SVD;

    // Define the matrix A (3x4) with 3 vertices and the extra 1s column
    let a = DMatrix::from_row_slice(
        3,
        4,
        &[
            point0.x, point0.y, point0.z, 1.0, // Vertex 1: (1, 2, 3, 1)
            point1.x, point1.y, point1.z, 1.0, // Vertex 2: (4, 5, 6, 1)
            point2.x, point2.y, point2.z, 1.0, // Vertex 3: (7, 8, 9, 1)
        ],
    );

    // Define the u-coordinates vector y (3x1)
    let u = DMatrix::from_column_slice(
        3,
        1,
        &[
            uv0, // u1
            uv1, // u2
            uv2, // u3
        ],
    );

    // Compute the SVD of A
    let svd = SVD::new(a.clone(), true, true);

    // Compute the pseudoinverse of A
    let a_pseudo = svd.pseudo_inverse(1e-6).expect("Pseudoinverse failed");

    // Solve for x using the pseudoinverse: x = A+ * y
    let x = &a_pseudo * u;

    return PlaneF {
        normal: Vector3 {
            x: x[0],
            y: x[1],
            z: x[2],
        },
        distance: x[3],
    };
}
