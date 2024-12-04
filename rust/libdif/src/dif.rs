use crate::ai_special_node::AISpecialNode;
use crate::force_field::ForceField;
use crate::game_entity::GameEntity;
use crate::interior::Interior;
use crate::interior_path_follower::InteriorPathFollower;
use crate::io::*;
use crate::trigger::Trigger;
use crate::types::*;
use crate::vehicle_collision::VehicleCollision;
use bytes::{Buf, BufMut};
use std::io::Cursor;

#[derive(Debug)]
pub struct Dif {
    pub interiors: Vec<Interior>,
    pub sub_objects: Vec<Interior>,
    pub triggers: Vec<Trigger>,
    pub interior_path_followers: Vec<InteriorPathFollower>,
    pub force_fields: Vec<ForceField>,
    pub ai_special_nodes: Vec<AISpecialNode>,
    pub vehicle_collision: Option<VehicleCollision>,
    pub game_entities: Vec<GameEntity>,
}

impl Dif {
    pub fn from_bytes<T>(from: T) -> DifResult<(Self, Version)>
    where
        T: AsRef<[u8]>,
    {
        let mut version = Version::new();
        let mut cursor = Cursor::new(from);
        let dif = Dif::read(&mut cursor, &mut version)?;
        Ok((dif, version))
    }
}

impl Readable<Dif> for Dif {
    fn read(from: &mut dyn Buf, version: &mut Version) -> DifResult<Self> {
        version.dif = u32::read(from, version)?;
        if version.dif != 44 {
            return Err(DifError::from("Bad version"));
        }

        if u8::read(from, version)? != 0 {
            let _ = PNG::read(from, version)?;
        }

        let interiors = Vec::<Interior>::read(from, version)?;
        let sub_objects = Vec::<Interior>::read(from, version)?;
        let triggers = Vec::<Trigger>::read(from, version)?;
        let interior_path_followers = Vec::<InteriorPathFollower>::read(from, version)?;
        let force_fields = Vec::<ForceField>::read(from, version)?;
        let ai_special_nodes = Vec::<AISpecialNode>::read(from, version)?;

        let vehicle_collision = if u32::read(from, version)? == 0 {
            None
        } else {
            Some(VehicleCollision::read(from, version)?)
        };

        let game_entities = if u32::read(from, version)? == 2 {
            Vec::<GameEntity>::read(from, version)?
        } else {
            vec![]
        };

        Ok(Dif {
            interiors,
            sub_objects,
            triggers,
            interior_path_followers,
            force_fields,
            ai_special_nodes,
            vehicle_collision,
            game_entities,
        })
    }
}

impl Writable<Dif> for Dif {
    fn write(&self, to: &mut dyn BufMut, version: &Version) -> DifResult<()> {
        version.dif.write(to, version)?;
        0u8.write(to, version)?;
        self.interiors.write(to, version)?;
        self.sub_objects.write(to, version)?;
        self.triggers.write(to, version)?;
        self.interior_path_followers.write(to, version)?;
        self.force_fields.write(to, version)?;
        self.ai_special_nodes.write(to, version)?;

        if let Some(vehicle_collision) = &self.vehicle_collision {
            1u32.write(to, version)?;
            vehicle_collision.write(to, version)?;
        } else {
            0u32.write(to, version)?;
        }

        if self.game_entities.len() > 0 {
            2u32.write(to, version)?;
            self.game_entities.write(to, version)?;
        } else {
            0u32.write(to, version)?;
        }
        0u32.write(to, version)?;

        Ok(())
    }
}
