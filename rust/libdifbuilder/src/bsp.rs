use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    sync::Mutex,
    vec,
};

use cgmath::{InnerSpace, Vector3};
use dif::types::{PlaneF, Point3F};
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};

use crate::builder::{OrdPlaneF, ProgressEventListener, Triangle};
use rayon::prelude::*;

#[derive(PartialEq, PartialOrd, Eq, Ord, Copy, Clone)]
pub enum SplitMethod {
    Fast,
    Exhaustive,
    None,
}

pub struct BSPConfig {
    pub split_method: SplitMethod,
    pub epsilon: f32,
}

pub static mut BSP_CONFIG: BSPConfig = BSPConfig {
    split_method: SplitMethod::Fast,
    epsilon: 1e-4,
};

#[derive(Clone)]
pub struct BSPPolygon {
    pub vertices: Vec<Point3F>,
    pub indices: Vec<usize>,
    pub plane: PlaneF,
    pub plane_id: usize,
    pub id: usize,
    pub used_plane: bool,
    pub inverted_plane: bool,
    pub area_calc: f32,
}

// (front, back, splits, coplanar, tiny_windings)
impl BSPPolygon {
    fn calculate_split_rating(
        &self,
        plane_id: usize,
        plane_list: &[PlaneF],
        considered_planes: &Mutex<RefCell<HashSet<usize>>>,
    ) -> (i32, i32, i32, i32, i32) {
        if !considered_planes
            .lock()
            .unwrap()
            .borrow()
            .contains(&plane_id)
        {
            if self.plane_id == plane_id {
                considered_planes
                    .lock()
                    .unwrap()
                    .borrow_mut()
                    .insert(plane_id);
                if self.inverted_plane {
                    return (1, 0, 0, 1, 0);
                } else {
                    return (0, 1, 0, 1, 0);
                }
            }
        }
        let unique_points = self
            .indices
            .iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        let test_plane = &plane_list[plane_id as usize];
        let mut max_front = 0.0;
        let mut min_back = 0.0;
        unique_points.iter().for_each(|p| {
            let pt = self.vertices[**p as usize];
            let d = test_plane.normal.dot(pt) + test_plane.distance;
            if d > max_front {
                max_front = d;
            }
            if d < min_back {
                min_back = d;
            }
        });
        let mut front = 0;
        let mut back = 0;
        let mut splits = 0;
        let mut tiny_windings = 0;
        if max_front > unsafe { BSP_CONFIG.epsilon } {
            front = 1;
        }
        if min_back < -unsafe { BSP_CONFIG.epsilon } {
            back = 1;
        }
        if max_front > unsafe { BSP_CONFIG.epsilon } && min_back < -unsafe { BSP_CONFIG.epsilon } {
            splits = 1;
        }
        if (max_front > 0.0 && max_front < 1.0) || (min_back < 0.0 && min_back > -1.0) {
            tiny_windings = 1;
        }
        (front, back, splits, 0, tiny_windings)
    }

    fn split(&self, plane: usize, plane_list: &[PlaneF]) -> [BSPPolygon; 2] {
        let mut front_brush = self.clone();
        let mut back_brush = self.clone();

        let plane_in_brush = self.plane_id == plane;

        back_brush.clip_plane(plane, plane_list, false);
        front_brush.clip_plane(plane, plane_list, true);

        let plane_in_front = front_brush.plane_id == plane;
        let plane_in_back = back_brush.plane_id == plane;

        if plane_in_brush {
            if !plane_in_back && !plane_in_front {
                assert!(false, "Wtf");
            }
            front_brush.used_plane = true;
            back_brush.used_plane = true;
        }

        return [front_brush, back_brush];
    }

    fn clip_plane(&mut self, plane: usize, plane_list: &[PlaneF], flip_face: bool) {
        let mut new_vertices = self.vertices.clone();
        let mut plane_value = plane_list[plane].clone();
        if flip_face {
            plane_value.normal *= -1.0;
            plane_value.distance *= -1.0;
        }

        let mut new_indices: Vec<usize> = vec![];
        let mut _points_on_plane = 0;
        for i in 0..self.indices.len() {
            let v1 = &self.vertices[self.indices[i] as usize];
            let v2 = &self.vertices[self.indices[(i + 1) % self.indices.len()] as usize];
            let d1 = v1.dot(plane_value.normal) + plane_value.distance;
            let d2 = v2.dot(plane_value.normal) + plane_value.distance;
            if d1 > unsafe { BSP_CONFIG.epsilon } {
                // Ignore
            }
            if d1 <= unsafe { BSP_CONFIG.epsilon } {
                // Keep
                new_indices.push(self.indices[i]);
            }
            if d1.abs() < unsafe { BSP_CONFIG.epsilon } {
                _points_on_plane += 1;
            }
            if (d1 > unsafe { BSP_CONFIG.epsilon } && d2 < -unsafe { BSP_CONFIG.epsilon })
                || (d1 < -unsafe { BSP_CONFIG.epsilon } && d2 > unsafe { BSP_CONFIG.epsilon })
            {
                let t = (-plane_value.distance - plane_value.normal.dot(*v1))
                    / plane_value.normal.dot(v2 - v1);
                let v3 = v1 + (v2 - v1) * t;
                new_indices.push(new_vertices.len());
                new_vertices.push(v3);
            }
        }
        // if clip_face && points_on_plane == face.indices.len() {
        //     new_indices.clear();
        // }
        // Sanity check
        let test_epsilon = unsafe { BSP_CONFIG.epsilon * 10.0 };
        for idx in new_indices.iter() {
            let pt = new_vertices[*idx as usize];
            let d = plane_value.normal.dot(pt) + plane_value.distance;
            if d > test_epsilon {
                assert!(false, "Invalid CLIP of {} (epsilon: {})", d, test_epsilon);
            }
        }

        self.vertices = new_vertices;
        self.indices = new_indices;
        self.area_calc = self.area();
    }

    fn _classify_score(&self, plane: &PlaneF) -> i32 {
        let mut front_count = 0;
        let mut back_count = 0;
        let mut on_count = 0;
        self.indices.iter().for_each(|i| {
            let pt = self.vertices[*i as usize];
            let face_dot = pt.dot(plane.normal) + plane.distance;
            if face_dot > unsafe { BSP_CONFIG.epsilon } {
                front_count += 1;
            } else if face_dot < unsafe { -BSP_CONFIG.epsilon } {
                back_count += 1;
            } else {
                on_count += 1;
            }
        });
        if front_count > 0 && back_count == 0 {
            front_count
        } else if front_count == 0 && back_count > 0 {
            -back_count
        } else if front_count == 0 && back_count == 0 && on_count > 0 {
            0
        } else {
            front_count - back_count
        }
    }

    fn classify_poly(&self, plane: &PlaneF) -> i32 {
        let mut front_count = 0;
        let mut back_count = 0;
        let mut on_count = 0;
        self.indices.iter().for_each(|i| {
            let pt = self.vertices[*i as usize];
            let face_dot = pt.dot(plane.normal) + plane.distance;
            if face_dot > unsafe { BSP_CONFIG.epsilon } {
                front_count += 1;
            } else if face_dot < unsafe { -BSP_CONFIG.epsilon } {
                back_count += 1;
            } else {
                on_count += 1;
            }
        });
        if front_count > 0 && back_count == 0 {
            1 // Is in front
        } else if front_count == 0 && back_count > 0 {
            -1 // Is in back
        } else if front_count == 0 && back_count == 0 && on_count > 0 {
            0 // Is on the plane
        } else {
            2 // Is spanning the plane
        }
    }

    fn area(&self) -> f32 {
        if self.indices.len() < 2 {
            0.0
        } else {
            let v0 = self.vertices[self.indices[0]];
            let mut a = 0.0;
            for i in 1..self.indices.len() {
                let v1 = self.vertices[self.indices[i]];
                let v2 = self.vertices[self.indices[(i + 1) % self.indices.len()]];
                let tri_a = (v1 - v0).cross(v2 - v0).magnitude() / 2.0;
                a += tri_a;
            }
            a
        }
    }
}

pub struct DIFBSPNode {
    pub brush_list: Vec<BSPPolygon>,
    pub front: Option<Box<DIFBSPNode>>,
    pub back: Option<Box<DIFBSPNode>>,
    pub plane_index: Option<usize>,
    pub solid: bool,
    pub avail_planes: Vec<usize>,
}

impl DIFBSPNode {
    fn from_brushes(brush_list: Vec<BSPPolygon>) -> DIFBSPNode {
        DIFBSPNode {
            front: None,
            back: None,
            plane_index: None,
            avail_planes: brush_list
                .iter()
                .map(|b| b.plane_id)
                .collect::<HashSet<_>>()
                .into_iter()
                .collect::<Vec<_>>(),
            brush_list: brush_list,
            solid: false,
        }
    }

    fn height(&self) -> i32 {
        let mut value = 0;
        if let Some(ref front) = self.front {
            value = std::cmp::max(value, front.height());
        }
        if let Some(ref back) = self.back {
            value = std::cmp::max(value, back.height());
        }
        value + 1
    }

    pub fn balance_factor(&self) -> i32 {
        let mut value = 0;
        if let Some(ref front) = self.front {
            value += front.height();
        }
        if let Some(ref back) = self.back {
            value -= back.height();
        }
        value
    }

    fn split(
        &mut self,
        plane_list: &[PlaneF],
        used_planes: &mut HashSet<usize>,
        depth: usize,
        progress_report_callback: &mut dyn ProgressEventListener,
    ) {
        let mut unused_planes = false;
        for brush in self.brush_list.iter() {
            if !brush.used_plane {
                unused_planes = true;
                break;
            }
        }
        let mut total_faces = 0;
        let mut remaining_faces = 0;
        for brush in self.brush_list.iter() {
            if !brush.used_plane {
                remaining_faces += 1;
            }
            total_faces += 1;
        }

        if unused_planes && self.plane_index == None {
            let split_plane = match unsafe { &BSP_CONFIG.split_method } {
                SplitMethod::Fast => self.select_best_splitter(plane_list),
                SplitMethod::Exhaustive => self.select_best_splitter_new(plane_list),
                _ => {
                    panic!("Should never reach here!")
                }
            };
            if let Some(split_plane) = split_plane {
                // Do split
                self.split_brush_list(split_plane, plane_list);
                self.plane_index = Some(split_plane);

                // if depth > 200 {
                //     println!(
                //         "Warning: depth over 200 {}, id {}, len {}",
                //         depth,
                //         self.plane_index.unwrap(),
                //         self.brush_list.len()
                //     );
                // }

                if !used_planes.contains(&split_plane) {
                    used_planes.insert(split_plane);
                    progress_report_callback.progress(
                        used_planes.len() as u32,
                        plane_list.len() as u32,
                        "Building BSP".to_string(),
                        "Built BSP".to_string(),
                    );
                }

                match self.front {
                    Some(ref mut n) => {
                        n.brush_list.iter_mut().for_each(|b| {
                            if b.plane_id == split_plane {
                                b.used_plane = true;
                            }
                        });
                        n.split(plane_list, used_planes, depth + 1, progress_report_callback);
                    }
                    None => {}
                };
                match self.back {
                    Some(ref mut n) => {
                        n.brush_list.iter_mut().for_each(|b| {
                            if b.plane_id == split_plane {
                                b.used_plane = true;
                            }
                        });
                        n.split(plane_list, used_planes, depth + 1, progress_report_callback);
                    }
                    None => {}
                };
            }
        }
    }

    fn split_brush_list(&mut self, plane_id: usize, plane_list: &[PlaneF]) {
        let mut front_brushes: Vec<BSPPolygon> = vec![];
        let mut back_brushes: Vec<BSPPolygon> = vec![];
        let mut front_solid = self.solid;
        let mut back_solid = self.solid;
        let mut plane_in_brush = false;
        for brush in self.brush_list.iter() {
            if brush.plane_id == plane_id {
                plane_in_brush = true;
                break;
            }
        }
        assert!(plane_in_brush, "Not in brush??");

        self.brush_list.iter().for_each(|b| {
            if b.plane_id == plane_id {
                let mut cl = b.clone();
                cl.used_plane = true;
                back_brushes.push(cl);
                back_solid = true;
            } else {
                let [front_brush, back_brush] = b.split(plane_id, plane_list);
                if front_brush.indices.len() > 2 {
                    front_solid = front_brush.used_plane;
                    front_brushes.push(front_brush);
                }
                if back_brush.indices.len() > 2 {
                    back_solid = back_brush.used_plane;
                    back_brushes.push(back_brush);
                }
            }
        });
        if front_brushes.len() != 0 {
            let front_node = DIFBSPNode {
                front: None,
                back: None,
                avail_planes: front_brushes
                    .iter()
                    .filter(|b| b.plane_id != plane_id && !b.used_plane)
                    .map(|b| b.plane_id)
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>(),
                brush_list: front_brushes,
                solid: front_solid,
                plane_index: None,
            };
            self.front = Some(Box::new(front_node));
        }
        if back_brushes.len() != 0 {
            let back_node = DIFBSPNode {
                front: None,
                back: None,
                solid: back_solid,
                avail_planes: back_brushes
                    .iter()
                    .filter(|b| b.plane_id != plane_id && !b.used_plane)
                    .map(|b| b.plane_id)
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>(),
                brush_list: back_brushes,
                plane_index: None,
            };
            self.back = Some(Box::new(back_node));
        }
        self.brush_list.clear();
        self.avail_planes.clear();
    }

    fn select_best_splitter_new(&self, plane_list: &[PlaneF]) -> Option<usize> {
        use std::f32::consts::PI;
        let mut vector_planes: Vec<(Vector3<f32>, Vec<usize>)> = vec![];
        // Create semi sphere unit vectors
        for i in 0..8 {
            for j in 0..8 {
                let p = -PI + PI * i as f32 / 8.0;
                let t = (PI / 2.0) * j as f32 / 8.0;
                let vecval = Vector3::new(t.cos() * p.sin(), t.sin() * p.sin(), p.cos());
                vector_planes.push((vecval, vec![]));
            }
        }
        // Quantize all the polygons to vectors according to max dot product
        let mut used_faces: HashSet<usize> = HashSet::new();
        self.brush_list.iter().for_each(|f| {
            if !f.used_plane && !used_faces.contains(&f.plane_id) {
                used_faces.insert(f.plane_id);
                let mut max_dot = -1.0;
                let mut max_index = None;
                let face_plane = &plane_list[f.plane_id];
                vector_planes.iter().enumerate().for_each(|(i, (v, _))| {
                    let dot = v.dot(face_plane.normal);
                    if dot > max_dot {
                        max_dot = dot;
                        max_index = Some(i);
                    }
                });
                if let Some(max_index) = max_index {
                    vector_planes[max_index].1.push(f.plane_id);
                }
            }
        });
        // Sort all the polygons from each list in vectorPlanes according to d
        for (_, p_list) in vector_planes.iter_mut() {
            p_list.sort_by(|a, b| plane_list[*a].distance.total_cmp(&plane_list[*b].distance));
        }

        // Get the least depth polygons from centre of each vectorPlanes
        let least_depth_planes = vector_planes
            .iter()
            .filter(|(_, p)| p.len() > 0)
            .map(|(_, pl)| pl[pl.len() / 2])
            .collect::<Vec<_>>();

        let val = least_depth_planes.par_iter().max_by_key(|&&p_idx| {
            self.calc_plane_rating(p_idx, plane_list)
            // self.brush_list
            //     .par_iter()
            //     .map(|b| b.classify_score(&plane_list[**p_idx]))
            //     .sum::<i32>()
        });

        // if let Some(&inner) = val {
        //     let entropy = self.calc_plane_rating(inner, plane_list);
        //     if entropy < 700 {
        //         println!("Warning: chose a plane {} with suboptimal entropy", inner);
        //     }
        // }

        match val {
            Some(i) => Some(*i),
            None => None,
        }
    }

    fn select_best_splitter(&self, plane_list: &[PlaneF]) -> Option<usize> {
        let mut rng = StdRng::seed_from_u64(42);

        let chosen_planes = self
            .brush_list
            .iter()
            .filter(|f| !f.used_plane)
            .map(|f| f.plane_id)
            .collect::<HashSet<_>>() // Get distinct
            .into_iter()
            .collect::<Vec<_>>();
        // let chosen_planes = &self.avail_planes;
        // Intersect this_planes and unused_planes
        let max_plane = chosen_planes
            .choose_multiple(&mut rng, 32)
            .collect::<Vec<_>>()
            .into_par_iter()
            .max_by_key(|&&p| self.calc_plane_rating(p, plane_list));

        match max_plane {
            Some(&x) => Some(x),
            None => None,
        }
    }

    fn calc_plane_rating(&self, plane_id: usize, plane_list: &[PlaneF]) -> i32 {
        let plane = &plane_list[plane_id as usize];
        let mut zero_count = 0;
        if plane.normal.x.abs() < unsafe { BSP_CONFIG.epsilon } {
            zero_count += 1;
        }
        if plane.normal.y.abs() < unsafe { BSP_CONFIG.epsilon } {
            zero_count += 1;
        }
        if plane.normal.z.abs() < unsafe { BSP_CONFIG.epsilon } {
            zero_count += 1;
        }
        let axial = zero_count == 2;
        let considered_planes = Mutex::from(RefCell::from(HashSet::new()));
        let (front, back, splits, coplanar, tiny_windings) = self
            .brush_list
            .par_iter()
            .map(|b| b.calculate_split_rating(plane_id, plane_list, &considered_planes))
            .reduce(
                || (0, 0, 0, 0, 0),
                |a, b| (a.0 + b.0, a.1 + b.1, a.2 + b.2, a.3 + b.3, a.4 + b.4),
            );

        // the rating adds to both when splits happens
        let front_only = front - splits;
        let back_only = back - splits;

        let real_front_and_back = front_only + back_only; // A symmetric_diff B

        let front_and_back = front + back; // A u B

        let jaccard = real_front_and_back as f32 / front_and_back as f32;

        let entropy = if front_and_back > 0 {
            (front as f32 / front_and_back as f32) * (front as f32 / front_and_back as f32).log2()
                + (back as f32 / front_and_back as f32)
                    * (back as f32 / front_and_back as f32).log2()
        } else {
            0.0
        };

        // let gini = 1.0
        //     - (front as f32 / front_and_back as f32).powi(2)
        //     - (back as f32 / front_and_back as f32).powi(2);

        // return (jaccard * 1000.0).round() as i32;

        return (-entropy * jaccard * 1000.0).round() as i32;

        // let mut final_score = 5 * coplanar - 5 * splits - (front - back).abs();
        // final_score -= 1000 * tiny_windings;
        // if axial {
        //     final_score += 5;
        // }
        // return final_score;
    }

    pub fn ray_cast(
        &self,
        start: Point3F,
        end: Point3F,
        plane_index: usize,
        plane_list: &[PlaneF],
    ) -> bool {
        if self.plane_index.is_none() {
            if self.solid {
                let mut found = false;
                for brush in self.brush_list.iter() {
                    if brush.plane_id == plane_index {
                        found = true;
                        break;
                    }
                    if found {
                        break;
                    }
                }
                return found;
            } else {
                false
            }
        } else {
            use std::cmp::Ordering;
            let plane_f = &plane_list[self.plane_index.unwrap()];
            let plane_norm = &plane_f.normal;
            let plane_d = &plane_f.distance;
            let s_side_value = plane_norm.dot(start) + plane_d;
            let e_side_value = plane_norm.dot(end) + plane_d;
            let s_side = s_side_value.total_cmp(&0.0);
            let e_side = e_side_value.total_cmp(&0.0);

            match (s_side, e_side) {
                (Ordering::Greater, Ordering::Greater)
                | (Ordering::Greater, Ordering::Equal)
                | (Ordering::Equal, Ordering::Greater) => {
                    if let Some(node_value) = &self.front {
                        node_value.ray_cast(start, end, plane_index, plane_list)
                    } else {
                        false
                    }
                }
                (Ordering::Greater, Ordering::Less) => {
                    let intersect_t =
                        (-plane_d - start.dot(*plane_norm)) / (end - start).dot(*plane_norm);
                    let ip = start + (end - start) * intersect_t;
                    if let Some(node_value) = &self.front {
                        if node_value.ray_cast(start, ip, plane_index, plane_list) {
                            return true;
                        }
                    }
                    if let Some(node_value) = &self.back {
                        node_value.ray_cast(ip, end, self.plane_index.unwrap(), plane_list)
                    } else {
                        false
                    }
                }
                (Ordering::Less, Ordering::Greater) => {
                    let intersect_t =
                        (-plane_d - start.dot(*plane_norm)) / (end - start).dot(*plane_norm);
                    let ip = start + (end - start) * intersect_t;
                    if let Some(node_value) = &self.back {
                        if node_value.ray_cast(start, ip, plane_index, plane_list) {
                            return true;
                        }
                    }
                    if let Some(node_value) = &self.front {
                        node_value.ray_cast(ip, end, self.plane_index.unwrap(), plane_list)
                    } else {
                        false
                    }
                }
                (Ordering::Less, Ordering::Less)
                | (Ordering::Less, Ordering::Equal)
                | (Ordering::Equal, Ordering::Less) => {
                    if let Some(node_value) = &self.back {
                        node_value.ray_cast(start, end, plane_index, plane_list)
                    } else {
                        false
                    }
                }
                _ => false,
            }
        }
    }

    fn split_new_impl(
        &mut self,
        plane_list: &[PlaneF],
        used_planes: &mut HashSet<usize>,
        depth: usize,
        progress_report_callback: &mut dyn ProgressEventListener,
    ) {
        let mut unused_planes = false;
        for brush in self.brush_list.iter() {
            if !brush.used_plane {
                unused_planes = true;
                break;
            }
        }
        let mut total_faces = 0;
        let mut remaining_faces = 0;
        for brush in self.brush_list.iter() {
            if !brush.used_plane {
                remaining_faces += 1;
            }
            total_faces += 1;
        }

        if unused_planes && self.plane_index == None {
            let split_plane = match unsafe { &BSP_CONFIG.split_method } {
                SplitMethod::Fast => self.select_best_splitter(plane_list),
                SplitMethod::Exhaustive => self.select_best_splitter_new(plane_list),
                _ => {
                    panic!("Should never reach here!")
                }
            };

            if let Some(split_plane) = split_plane {
                self.plane_index = Some(split_plane);

                // Classify each brush as front, back, or coinciding
                let mut front_brushes: Vec<BSPPolygon> = vec![];
                let mut back_brushes: Vec<BSPPolygon> = vec![];

                self.brush_list.iter().for_each(|b| {
                    if b.plane_id == split_plane {
                        // Coinciding, put in back for now
                        let mut cl = b.clone();
                        cl.used_plane = true;
                        back_brushes.push(cl);
                    } else {
                        let classify_score = b.classify_poly(&plane_list[split_plane]);

                        if classify_score == 1 {
                            front_brushes.push(b.clone());
                        } else if classify_score == -1 {
                            back_brushes.push(b.clone());
                        } else if classify_score == 0 {
                            // Coinciding, put in back for now
                            let mut cl = b.clone();
                            cl.used_plane = true;
                            back_brushes.push(cl);
                        } else if classify_score == 2 {
                            // Spanning, split it
                            let [front_brush, back_brush] = b.split(split_plane, plane_list);
                            if front_brush.indices.len() > 2 {
                                front_brushes.push(front_brush);
                            }
                            if back_brush.indices.len() > 2 {
                                back_brushes.push(back_brush);
                            }
                        }
                    }
                });

                if !used_planes.contains(&split_plane) {
                    used_planes.insert(split_plane);
                    progress_report_callback.progress(
                        used_planes.len() as u32,
                        plane_list.len() as u32,
                        "Building BSP".to_string(),
                        "Built BSP".to_string(),
                    );
                }

                if front_brushes.len() != 0 {
                    let front_node = DIFBSPNode {
                        front: None,
                        back: None,
                        avail_planes: front_brushes
                            .iter()
                            .filter(|b| b.plane_id != split_plane && !b.used_plane)
                            .map(|b| b.plane_id)
                            .collect::<HashSet<_>>()
                            .into_iter()
                            .collect::<Vec<_>>(),
                        brush_list: front_brushes,
                        solid: false,
                        plane_index: None,
                    };
                    self.front = Some(Box::new(front_node));
                }
                if back_brushes.len() != 0 {
                    let back_node = DIFBSPNode {
                        front: None,
                        back: None,
                        solid: false,
                        avail_planes: back_brushes
                            .iter()
                            .filter(|b| b.plane_id != split_plane && !b.used_plane)
                            .map(|b| b.plane_id)
                            .collect::<HashSet<_>>()
                            .into_iter()
                            .collect::<Vec<_>>(),
                        brush_list: back_brushes,
                        plane_index: None,
                    };
                    self.back = Some(Box::new(back_node));
                }

                self.brush_list.clear();
                self.avail_planes.clear();

                if let Some(ref mut n) = self.front {
                    n.brush_list.iter_mut().for_each(|b| {
                        if b.plane_id == split_plane {
                            b.used_plane = true;
                        }
                    });
                    n.split_new_impl(plane_list, used_planes, depth + 1, progress_report_callback);
                };
                if let Some(ref mut n) = self.back {
                    n.brush_list.iter_mut().for_each(|b| {
                        if b.plane_id == split_plane {
                            b.used_plane = true;
                        }
                    });
                    n.split_new_impl(plane_list, used_planes, depth + 1, progress_report_callback);
                };
            }
        }
    }
}

pub fn build_bsp(
    brush_list: &[Triangle],
    progress_report_callback: &mut dyn ProgressEventListener,
) -> (DIFBSPNode, Vec<PlaneF>) {
    let mut plane_map: HashMap<OrdPlaneF, usize> = HashMap::new();
    let mut plane_list: Vec<PlaneF> = vec![];

    let bsp_polygons = brush_list
        .iter()
        .map(|b| {
            let mut plane_id = plane_list.len();
            let ord_plane = OrdPlaneF::from(&b.plane);
            let mut plane_inverted = false;
            if plane_map.contains_key(&ord_plane) {
                plane_id = plane_map[&ord_plane];
            } else {
                // Try inverted
                // let mut pinvplane = b.plane.clone();
                // pinvplane.normal *= -1.0;
                // pinvplane.distance *= -1.0;
                // let ord_plane = OrdPlaneF::from(&pinvplane);
                // if plane_map.contains_key(&ord_plane) {
                //     plane_id = plane_map[&ord_plane];
                //     plane_inverted = true;
                // } else {
                plane_list.push(b.plane.clone());
                plane_map.insert(OrdPlaneF::from(&b.plane), plane_id);
                // }
            }

            let mut poly = BSPPolygon {
                id: b.id as usize,
                plane: b.plane.clone(),
                plane_id: plane_id,
                indices: vec![0, 1, 2],
                vertices: b.verts.to_vec(),
                used_plane: false,
                inverted_plane: plane_inverted,
                area_calc: 0.0,
            };
            poly.area_calc = poly.area();
            poly
        })
        .collect::<Vec<_>>();

    let mut root = DIFBSPNode::from_brushes(bsp_polygons);
    if unsafe { BSP_CONFIG.split_method } == SplitMethod::None {
        root.front = Some(Box::new(DIFBSPNode {
            back: None,
            brush_list: Vec::new(),
            front: None,
            plane_index: None,
            solid: false,
            avail_planes: Vec::new(),
        }));
        root.back = Some(Box::new(DIFBSPNode {
            back: None,
            brush_list: Vec::new(),
            front: None,
            plane_index: None,
            solid: false,
            avail_planes: Vec::new(),
        }));
        root.plane_index = Some(0);
    } else {
        let mut used_planes: HashSet<usize> = HashSet::new();
        root.split_new_impl(&plane_list, &mut used_planes, 0, progress_report_callback);
    }
    (root, plane_list)
}
