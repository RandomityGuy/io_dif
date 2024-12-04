pub mod bsp;
pub mod builder;
use std::io::Cursor;

use bsp::BSP_CONFIG;
use builder::{BSPReport, ProgressEventListener};
use builder::{PLANE_EPSILON, POINT_EPSILON};
use dif::io::EngineVersion;
use dif::io::Version;
use quick_xml::de::Deserializer;
use serde::Deserialize;

use crate::bsp::SplitMethod;

static mut MB_ONLY: bool = true;

pub unsafe fn set_convert_configuration(
    mb_only: bool,
    point_epsilon: f32,
    plane_epsilon: f32,
    split_epsilon: f32,
    split_method: SplitMethod,
) {
    unsafe {
        BSP_CONFIG.epsilon = split_epsilon;
        BSP_CONFIG.split_method = split_method;
        POINT_EPSILON = point_epsilon;
        PLANE_EPSILON = plane_epsilon;
        MB_ONLY = mb_only;
    }
}

// pub fn convert_to_dif(
//     engine_ver: EngineVersion,
//     interior_version: u32,
//     progress_fn: &mut dyn ProgressEventListener,
// ) -> (Vec<Vec<u8>>, Vec<BSPReport>) {
//     let version = Version {
//         engine: engine_ver,
//         dif: 44,
//         interior: interior_version,
//         material_list: 1,
//         vehicle_collision: 0,
//         force_field: 0,
//     };
//     let b = builder::DIFBuilder::new(true);
// }

// pub fn convert_csx_to_dif(
//     csxbuf: String,
//     engine_ver: EngineVersion,
//     interior_version: u32,
//     progress_fn: &mut dyn ProgressEventListener,
// ) -> (Vec<Vec<u8>>, Vec<BSPReport>) {
//     let cur = Cursor::new(csxbuf);
//     let reader = std::io::BufReader::new(cur);
//     let mut des = Deserializer::from_reader(reader);
//     let mut cscene = csx::ConstructorScene::deserialize(&mut des).unwrap();

//     // Transform the vertices and planes to absolute coords, also assign unique ids to face
//     preprocess_csx(&mut cscene);
//     let version = Version {
//         engine: engine_ver,
//         dif: 44,
//         interior: interior_version,
//         material_list: 1,
//         vehicle_collision: 0,
//         force_field: 0,
//     };
//     let buf = convert_csx(&cscene, version, unsafe { MB_ONLY }, progress_fn);
//     buf
// }
