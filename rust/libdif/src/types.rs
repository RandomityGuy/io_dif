use crate::io::*;
use crate::io::{Readable, Writable};
use bytes::{Buf, BufMut};
use cgmath::{InnerSpace, Matrix, Matrix4, Quaternion, Vector2, Vector3};
use dif_derive::{Readable, Writable};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::string::FromUtf8Error;

pub type Point2F = Vector2<f32>;

pub type Point2I = Vector2<i32>;

pub type Point3F = Vector3<f32>;

#[derive(Debug, Readable, Writable, Clone)]
pub struct BoxF {
    pub min: Point3F,
    pub max: Point3F,
}

#[derive(Debug, Readable, Writable, Clone)]
pub struct SphereF {
    pub origin: Point3F,
    pub radius: f32,
}

#[derive(Debug, Readable, Writable, Clone)]
pub struct PlaneF {
    pub normal: Point3F,
    pub distance: f32,
}

pub type QuatF = Quaternion<f32>;

#[derive(Clone, Copy, Debug, Readable, Writable)]
pub struct ColorI {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub type MatrixF = Matrix4<f32>;

pub type Dictionary = HashMap<String, String>;

#[derive(Debug, Clone)]
pub struct PNG {
    pub data: Vec<u8>,
}

pub type DifResult<T> = Result<T, DifError>;

#[derive(Debug)]
pub struct DifError {
    pub message: String,
}

impl BoxF {
    pub fn center(&self) -> Point3F {
        (self.min + self.max) / 2.0
    }
    pub fn extent(&self) -> Point3F {
        self.max - self.min
    }
    pub fn union(&self, other: &BoxF) -> BoxF {
        BoxF {
            min: Vector3 {
                x: self.min.x.min(other.min.x),
                y: self.min.y.min(other.min.y),
                z: self.min.z.min(other.min.z),
            },
            max: Vector3 {
                x: self.max.x.max(other.max.x),
                y: self.max.y.max(other.max.y),
                z: self.max.z.max(other.max.z),
            },
        }
    }
    pub fn union_point(&self, other: &Point3F) -> BoxF {
        BoxF {
            min: Vector3 {
                x: self.min.x.min(other.x),
                y: self.min.y.min(other.y),
                z: self.min.z.min(other.z),
            },
            max: Vector3 {
                x: self.max.x.max(other.x),
                y: self.max.y.max(other.y),
                z: self.max.z.max(other.z),
            },
        }
    }
    pub fn contains(&self, point: &Point3F) -> bool {
        return point.x >= self.min.x
            && point.y >= self.min.y
            && point.z >= self.min.z
            && point.x <= self.max.x
            && point.y <= self.max.y
            && point.z <= self.max.z;
    }

    pub fn from_vertices(vertices: &[&Point3F]) -> Self {
        use std::f32::{INFINITY, NEG_INFINITY};
        let mut b = BoxF {
            min: Vector3 {
                x: INFINITY,
                y: INFINITY,
                z: INFINITY,
            },
            max: Vector3 {
                x: NEG_INFINITY,
                y: NEG_INFINITY,
                z: NEG_INFINITY,
            },
        };

        for vertex in vertices {
            b.min.x = b.min.x.min(vertex.x);
            b.min.y = b.min.y.min(vertex.y);
            b.min.z = b.min.z.min(vertex.z);
            b.max.x = b.max.x.max(vertex.x);
            b.max.y = b.max.y.max(vertex.y);
            b.max.z = b.max.z.max(vertex.z);
        }

        b
    }
}

impl PlaneF {
    pub fn from_triangle(v0: Point3F, v1: Point3F, v2: Point3F) -> Self {
        let normal = (v2 - v0).cross(v1 - v0).normalize();

        //Use the center of the plane, probably correct
        let average_point = (v0 + v1 + v2) / 3.0;

        let distance = (-average_point).dot(normal);
        PlaneF { normal, distance }
    }
}

impl From<&'static str> for DifError {
    fn from(message: &'static str) -> Self {
        DifError {
            message: message.into(),
        }
    }
}

impl From<FromUtf8Error> for DifError {
    fn from(err: FromUtf8Error) -> Self {
        DifError {
            message: format!("UTF-8 Error: {}", err),
        }
    }
}

impl Display for DifError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "DIF error: {}", self.message)
    }
}

impl Error for DifError {
    fn description(&self) -> &str {
        "DIF Error"
    }
}

impl Readable<Point2F> for Point2F {
    fn read(from: &mut dyn Buf, version: &mut Version) -> DifResult<Self> {
        Ok(Point2F {
            x: f32::read(from, version)?,
            y: f32::read(from, version)?,
        })
    }
}

impl Writable<Point2F> for Point2F {
    fn write(&self, to: &mut dyn BufMut, version: &Version) -> DifResult<()> {
        self.x.write(to, version)?;
        self.y.write(to, version)?;
        Ok(())
    }
}

impl Readable<Point2I> for Point2I {
    fn read(from: &mut dyn Buf, version: &mut Version) -> DifResult<Self> {
        Ok(Point2I {
            x: i32::read(from, version)?,
            y: i32::read(from, version)?,
        })
    }
}

impl Writable<Point2I> for Point2I {
    fn write(&self, to: &mut dyn BufMut, version: &Version) -> DifResult<()> {
        self.x.write(to, version)?;
        self.y.write(to, version)?;
        Ok(())
    }
}

impl Readable<Point3F> for Point3F {
    fn read(from: &mut dyn Buf, version: &mut Version) -> DifResult<Self> {
        Ok(Point3F {
            x: f32::read(from, version)?,
            y: f32::read(from, version)?,
            z: f32::read(from, version)?,
        })
    }
}

impl Writable<Point3F> for Point3F {
    fn write(&self, to: &mut dyn BufMut, version: &Version) -> DifResult<()> {
        self.x.write(to, version)?;
        self.y.write(to, version)?;
        self.z.write(to, version)?;
        Ok(())
    }
}

impl Readable<QuatF> for QuatF {
    fn read(from: &mut dyn Buf, version: &mut Version) -> DifResult<Self> {
        Ok(QuatF {
            s: f32::read(from, version)?,
            v: Vector3 {
                x: f32::read(from, version)?,
                y: f32::read(from, version)?,
                z: f32::read(from, version)?,
            },
        })
    }
}

impl Writable<QuatF> for QuatF {
    fn write(&self, to: &mut dyn BufMut, version: &Version) -> DifResult<()> {
        self.s.write(to, version)?;
        self.v.write(to, version)?;
        Ok(())
    }
}

impl Readable<MatrixF> for MatrixF {
    fn read(from: &mut dyn Buf, version: &mut Version) -> DifResult<MatrixF> {
        let m = MatrixF::new(
            f32::read(from, version)?,
            f32::read(from, version)?,
            f32::read(from, version)?,
            f32::read(from, version)?,
            f32::read(from, version)?,
            f32::read(from, version)?,
            f32::read(from, version)?,
            f32::read(from, version)?,
            f32::read(from, version)?,
            f32::read(from, version)?,
            f32::read(from, version)?,
            f32::read(from, version)?,
            f32::read(from, version)?,
            f32::read(from, version)?,
            f32::read(from, version)?,
            f32::read(from, version)?,
        );
        Ok(m.transpose())
    }
}

impl Writable<MatrixF> for MatrixF {
    fn write(&self, to: &mut dyn BufMut, version: &Version) -> DifResult<()> {
        self.x.x.write(to, version)?;
        self.y.x.write(to, version)?;
        self.z.x.write(to, version)?;
        self.w.x.write(to, version)?;
        self.x.y.write(to, version)?;
        self.y.y.write(to, version)?;
        self.z.y.write(to, version)?;
        self.w.y.write(to, version)?;
        self.x.z.write(to, version)?;
        self.y.z.write(to, version)?;
        self.z.z.write(to, version)?;
        self.w.z.write(to, version)?;
        self.x.w.write(to, version)?;
        self.y.w.write(to, version)?;
        self.z.w.write(to, version)?;
        self.w.w.write(to, version)?;
        Ok(())
    }
}

impl Readable<Dictionary> for Dictionary {
    fn read(from: &mut dyn Buf, version: &mut Version) -> DifResult<Self> {
        let length = u32::read(from, version)?;

        let mut result = Dictionary::new();

        for _ in 0..length {
            let name = String::read(from, version)?;
            let value = String::read(from, version)?;
            result.insert(name, value);
        }

        Ok(result)
    }
}

impl Writable<Dictionary> for Dictionary {
    fn write(&self, to: &mut dyn BufMut, version: &Version) -> DifResult<()> {
        (self.len() as u32).write(to, version)?;

        for (name, value) in self {
            name.write(to, version)?;
            value.write(to, version)?;
        }

        Ok(())
    }
}

impl Readable<PNG> for PNG {
    fn read(from: &mut dyn Buf, version: &mut Version) -> DifResult<Self> {
        let footer = [0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82];
        let mut data = vec![];
        while data.len() < 8 || !data.ends_with(&footer) {
            data.push(u8::read(from, version)?);
        }
        Ok(PNG { data })
    }
}

impl Writable<PNG> for PNG {
    fn write(&self, to: &mut dyn BufMut, version: &Version) -> DifResult<()> {
        for byte in &self.data {
            byte.write(to, version)?;
        }

        Ok(())
    }
}
