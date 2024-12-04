// C library

use std::{
    collections::HashMap,
    ffi::{c_char, CStr, CString},
    io::Cursor,
    sync::Arc,
    thread,
    time::Instant,
};

use ::dif::types::Point3F;
use cgmath::Quaternion;
use dif::{
    dif::Dif,
    game_entity::GameEntity,
    interior::Interior,
    interior_path_follower::{InteriorPathFollower, WayPoint},
    io::{Version, Writable},
    types::{Dictionary, Point2F},
};
use difbuilder::{
    builder::{self, ProgressEventListener, Triangle},
    set_convert_configuration,
};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

struct ConsoleProgressListener {
    thread_tx: Option<std::sync::mpsc::Sender<(bool, u32, u32, String, String)>>,
    listener_cb: unsafe extern "C" fn(bool, u32, u32, *const c_char, *const c_char),
}

impl ConsoleProgressListener {
    fn new(
        listener_cb: unsafe extern "C" fn(bool, u32, u32, *const c_char, *const c_char),
    ) -> Self {
        ConsoleProgressListener {
            thread_tx: None,
            listener_cb,
        }
    }
    fn init(&mut self) -> thread::JoinHandle<()> {
        let (sender, receiver) = std::sync::mpsc::channel();
        self.thread_tx = Some(sender);
        let handler: thread::JoinHandle<_> = thread::spawn(move || {
            let progress_bar: MultiProgress = MultiProgress::new();
            let mut progress_types: HashMap<String, (ProgressBar, Instant)> = HashMap::new();
            loop {
                let (stop, current, total, status, finish_status): (
                    bool,
                    u32,
                    u32,
                    String,
                    String,
                ) = receiver.recv().unwrap();
                if stop {
                    break;
                }
                if total == 0 {
                    progress_bar.println(status).unwrap();
                    progress_bar.clear().unwrap();
                } else if let Some((bar, ref mut last_updated)) = progress_types.get_mut(&status) {
                    let recvtime = std::time::Instant::now();
                    if recvtime.duration_since(*last_updated).as_millis() < 100 && total != current
                    {
                        continue;
                    }
                    *last_updated = recvtime;

                    bar.set_length(total as u64);
                    bar.set_position(current as u64);
                    bar.set_message(status.clone());
                    if current == total {
                        bar.finish_with_message(finish_status);
                        // self.progress_types.remove(&status);
                    }
                } else {
                    let sty =
                        ProgressStyle::with_template("{msg} {bar:40.cyan/blue} {pos:>7}/{len:7}")
                            .unwrap();
                    let pbar = progress_bar.add(ProgressBar::new(total as u64));
                    pbar.set_style(sty);
                    pbar.set_position(current as u64);
                    pbar.set_message(status.clone());
                    progress_types.insert(status.clone(), (pbar, std::time::Instant::now()));
                }
            }
        });
        handler
    }

    fn stop(&self) {
        unsafe {
            let stat = CString::new("").unwrap();
            let fin = CString::new("").unwrap();
            (self.listener_cb)(false, 0, 0, stat.as_ptr(), fin.as_ptr());
        }
        self.thread_tx
            .as_ref()
            .unwrap()
            .send((true, 0, 0, "".to_owned(), "".to_owned()))
            .unwrap();
    }
}

impl ProgressEventListener for ConsoleProgressListener {
    fn progress(&mut self, current: u32, total: u32, status: String, finish_status: String) {
        unsafe {
            let stat = CString::new(status.clone()).unwrap();
            let fin = CString::new(finish_status.clone()).unwrap();
            (self.listener_cb)(false, current, total, stat.as_ptr(), fin.as_ptr());
        }
        self.thread_tx
            .as_ref()
            .unwrap()
            .send((false, current, total, status, finish_status))
            .unwrap();
    }
}

pub struct TriangleRaw {
    verts: [Point3F; 3],
    uv: [Point2F; 3],
    norm: Point3F,
    material: String,
}

pub struct PathedInteriorImpl {
    pub interior: Interior,
    pub waypoints: Vec<WayPoint>,
}

pub struct DifBuilderImpl {
    pub triangles: Vec<TriangleRaw>,
    pub pathed_interiors: Vec<PathedInteriorImpl>,
}

pub struct MarkerListImpl {
    pub markers: Vec<WayPoint>,
}

#[no_mangle]
pub extern "C" fn new_difbuilder() -> *const DifBuilderImpl {
    Arc::into_raw(Arc::new(DifBuilderImpl {
        triangles: Vec::new(),
        pathed_interiors: Vec::new(),
    }))
}

#[no_mangle]
pub unsafe extern "C" fn dispose_difbuilder(ptr: *const DifBuilderImpl) {
    Arc::decrement_strong_count(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn dispose_dif(ptr: *const Dif) {
    Arc::decrement_strong_count(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn new_dict() -> *const Dictionary {
    Arc::into_raw(Arc::new(Dictionary::new()))
}

#[no_mangle]
pub unsafe extern "C" fn dispose_dict(ptr: *const Dictionary) {
    Arc::decrement_strong_count(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn add_dict_kvp(
    ptr: *mut Dictionary,
    key: *const c_char,
    value: *const c_char,
) {
    ptr.as_mut().unwrap().insert(
        CStr::from_ptr(key).to_str().unwrap().to_owned(),
        CStr::from_ptr(value).to_str().unwrap().to_owned(),
    );
}

#[no_mangle]
pub unsafe extern "C" fn new_marker_list() -> *const MarkerListImpl {
    Arc::into_raw(Arc::new(MarkerListImpl {
        markers: Vec::new(),
    }))
}

#[no_mangle]
pub unsafe extern "C" fn dispose_marker_list(ptr: *const MarkerListImpl) {
    Arc::decrement_strong_count(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn push_marker(
    ptr: *mut MarkerListImpl,
    pos: *const f32,
    ms_to_next: i32,
    initial_target_position: i32,
) {
    ptr.as_mut().unwrap().markers.push(WayPoint {
        ms_to_next: ms_to_next as u32,
        position: Point3F::new(*pos, *pos.offset(1), *pos.offset(2)),
        smoothing_type: 0,
        rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
    });
}

#[no_mangle]
pub unsafe extern "C" fn add_game_entity(
    ptr: *mut Dif,
    game_class: *const c_char,
    datablock: *const c_char,
    pos: *const f32,
    dict: *const Dictionary,
) {
    ptr.as_mut().unwrap().game_entities.push(GameEntity {
        datablock: CStr::from_ptr(datablock).to_str().unwrap().to_owned(),
        game_class: CStr::from_ptr(game_class).to_str().unwrap().to_owned(),
        position: Point3F::new(*pos, *pos.offset(1), *pos.offset(2)),
        properties: dict.as_ref().unwrap().clone(),
    });
}

#[no_mangle]
pub unsafe extern "C" fn add_triangle(
    ptr: *mut DifBuilderImpl,
    p1: *const f32,
    p2: *const f32,
    p3: *const f32,
    uv1: *const f32,
    uv2: *const f32,
    uv3: *const f32,
    normal: *const f32,
    material: *const c_char,
) {
    ptr.as_mut().unwrap().triangles.push(TriangleRaw {
        verts: [
            Point3F::new(*p1, *p1.offset(1), *p1.offset(2)),
            Point3F::new(*p2, *p2.offset(1), *p2.offset(2)),
            Point3F::new(*p3, *p3.offset(1), *p3.offset(2)),
        ],
        uv: [
            Point2F::new(*uv1, *uv1.offset(1)),
            Point2F::new(*uv2, *uv2.offset(1)),
            Point2F::new(*uv3, *uv3.offset(1)),
        ],
        material: CStr::from_ptr(material).to_str().unwrap().to_owned(),
        norm: Point3F::new(*normal, *normal.offset(1), *normal.offset(2)),
    });
}

#[no_mangle]
pub unsafe extern "C" fn add_trigger(
    ptr: *mut DifBuilderImpl,
    pos: *const f32,
    name: *const u8,
    datablock: *const u8,
    props: *const Dictionary,
) {
    // Do nothing
}

#[no_mangle]
pub unsafe extern "C" fn add_pathed_interior(
    ptr: *mut DifBuilderImpl,
    dif: *mut Dif,
    marker_list: *const MarkerListImpl,
) {
    let mut pathed_interior = PathedInteriorImpl {
        interior: dif.as_mut().unwrap().interiors.swap_remove(0),
        waypoints: marker_list.as_ref().unwrap().markers.clone(),
    };
    ptr.as_mut().unwrap().pathed_interiors.push(pathed_interior);
}

#[no_mangle]
pub unsafe extern "C" fn build(
    ptr: *mut DifBuilderImpl,
    mb_only: bool,
    bsp_mode: i32,
    point_epsilon: f32,
    plane_epsilon: f32,
    split_epsilon: f32,
    listener_cb: unsafe extern "C" fn(bool, u32, u32, *const c_char, *const c_char),
) -> *const Dif {
    let mut listener = ConsoleProgressListener::new(listener_cb);
    let join_handler = listener.init();

    set_convert_configuration(
        mb_only,
        point_epsilon,
        plane_epsilon,
        split_epsilon,
        match bsp_mode {
            0 => difbuilder::bsp::SplitMethod::Fast,
            1 => difbuilder::bsp::SplitMethod::Exhaustive,
            _ => difbuilder::bsp::SplitMethod::None,
        },
    );

    let mut actual_builder = builder::DIFBuilder::new(true);
    for tri in ptr.as_ref().unwrap().triangles.iter() {
        actual_builder.add_triangle(
            tri.verts[0],
            tri.verts[1],
            tri.verts[2],
            tri.uv[0],
            tri.uv[1],
            tri.uv[2],
            tri.norm,
            tri.material.clone(),
        );
    }

    let (itr, r) = actual_builder.build(&mut listener);

    listener.stop();
    join_handler.join().unwrap();
    // Write the report
    println!("BSP Report");
    println!(
        "Raycast Coverage: {}/{} ({}% of surface area)",
        r.hit, r.total, r.hit_area_percentage
    );
    println!("Balance Factor: {}", r.balance_factor);

    let mut dif = dif_with_interiors(vec![itr]);
    // Add the pathed interiors
    for pathed_interior in ptr.as_ref().unwrap().pathed_interiors.iter() {
        let sub_index = dif.sub_objects.len();
        dif.sub_objects.push(pathed_interior.interior.clone());
        dif.interior_path_followers.push(InteriorPathFollower {
            datablock: "PathedDefault".to_owned(),
            interior_res_index: sub_index as u32,
            name: "MustChange".to_owned(),
            offset: Point3F::new(0.0, 0.0, 0.0),
            total_ms: pathed_interior
                .waypoints
                .iter()
                .map(|wp| wp.ms_to_next)
                .sum(),
            way_points: pathed_interior.waypoints.clone(),
            trigger_ids: vec![],
            properties: Dictionary::new(),
        });
    }

    Arc::into_raw(Arc::new(dif))
}

#[no_mangle]
pub unsafe extern "C" fn write_dif(dif: *const Dif, path: *const c_char) {
    let version = Version {
        engine: dif::io::EngineVersion::MBG,
        dif: 44,
        interior: 0,
        material_list: 1,
        vehicle_collision: 0,
        force_field: 0,
    };
    let mut buf = vec![];
    dif.as_ref().unwrap().write(&mut buf, &version).unwrap();
    let path = CStr::from_ptr(path).to_str().unwrap();
    std::fs::write(path, buf).unwrap();
}

pub fn dif_with_interiors(interiors: Vec<Interior>) -> Dif {
    Dif {
        interiors,
        sub_objects: vec![],
        triggers: vec![],
        interior_path_followers: vec![],
        force_fields: vec![],
        ai_special_nodes: vec![],
        vehicle_collision: None,
        game_entities: vec![],
    }
}
