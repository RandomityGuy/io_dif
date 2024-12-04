use crate::types::*;
use bytes::{Buf, BufMut};
use std::mem::size_of;
use typed_ints::TypedInt;

#[derive(PartialEq, Eq, Debug)]
pub enum EngineVersion {
    Unknown,
    MBG,
    TGE,
    TGEA,
    T3D,
}

pub struct Version {
    pub engine: EngineVersion,
    pub dif: u32,
    pub interior: u32,
    pub material_list: u8,
    pub vehicle_collision: u32,
    pub force_field: u32,
}

impl Version {
    pub fn new() -> Version {
        Version {
            engine: EngineVersion::Unknown,
            dif: 0,
            interior: 0,
            material_list: 0,
            vehicle_collision: 0,
            force_field: 0,
        }
    }

    pub fn is_tge(&self) -> bool {
        match self.engine {
            EngineVersion::MBG | EngineVersion::TGE => true,
            _ => false,
        }
    }
}

pub trait Readable<T> {
    fn read(from: &mut dyn Buf, version: &mut Version) -> DifResult<T>;
}

pub trait Writable<T> {
    fn write(&self, to: &mut dyn BufMut, version: &Version) -> DifResult<()>;
}

pub fn read_vec<T, T2>(
    from: &mut dyn Buf,
    version: &mut Version,
    compare_func: fn(bool, u8) -> bool,
    into_func: fn(T2) -> T,
) -> DifResult<Vec<T>>
where
    T: Readable<T>,
    T2: Readable<T2>,
{
    let mut length = u32::read(from, version)?;

    let mut signed = false;
    let mut param = 0u8;

    if (length & 0x80000000) != 0 {
        length ^= 0x80000000;
        signed = true;
        param = u8::read(from, version)?;
    }

    let mut result: Vec<T> = Vec::with_capacity(length as usize);

    for _ in 0..length {
        if compare_func(signed, param) {
            result.push(into_func(T2::read(from, version)?));
        } else {
            result.push(T::read(from, version)?);
        }
    }

    Ok(result)
}

pub fn write_vec<'a, T: 'a, T2: 'a>(
    vec: &'a Vec<T>,
    to: &mut dyn BufMut,
    version: &Version,
) -> DifResult<()>
where
    T2: Writable<T2>,
    &'a T: Into<&'a T2>,
{
    (vec.len() as u32).write(to, version)?;
    for item in vec {
        let converted: &'a T2 = item.into();
        converted.write(to, version)?;
    }

    Ok(())
}

pub fn read_vec_fn<T, F>(
    from: &mut dyn Buf,
    version: &mut Version,
    read_func: F,
) -> DifResult<Vec<T>>
where
    F: Fn(&mut dyn Buf, &mut Version) -> DifResult<T>,
{
    let mut length = u32::read(from, version)?;

    if (length & 0x80000000) != 0 {
        length ^= 0x80000000;
        u8::read(from, version)?;
    }

    let mut result: Vec<T> = Vec::with_capacity(length as usize);

    for _ in 0..length {
        result.push(read_func(from, version)?);
    }

    Ok(result)
}

pub fn write_vec_fn<'a, T: 'a, T2: 'a>(
    vec: &'a Vec<T>,
    to: &mut dyn BufMut,
    version: &Version,
    convert_fn: fn(&'a T) -> T2,
) -> DifResult<()>
where
    T2: Writable<T2>,
{
    (vec.len() as u32).write(to, version)?;
    for item in vec {
        let converted: T2 = convert_fn(item);
        converted.write(to, version)?;
    }

    Ok(())
}

pub fn read_vec_extra<T, T2>(
    from: &mut dyn Buf,
    version: &mut Version,
    extra_func: fn(&mut dyn Buf, &mut Version) -> DifResult<T2>,
) -> DifResult<(Vec<T>, T2)>
where
    T: Readable<T>,
{
    let length = u32::read(from, version)?;
    let extra = extra_func(from, version)?;
    let mut result: Vec<T> = Vec::with_capacity(length as usize);

    for _ in 0..length {
        result.push(T::read(from, version)?);
    }

    Ok((result, extra))
}

pub fn write_vec_extra<'a, T: 'a>(
    vec: &'a Vec<T>,
    to: &mut dyn BufMut,
    version: &Version,
    extra_func: impl Fn(&mut dyn BufMut, &Version) -> DifResult<()>,
) -> DifResult<()>
where
    T: Writable<T>,
{
    (vec.len() as u32).write(to, version)?;
    extra_func(to, version)?;

    for item in vec {
        item.write(to, version)?;
    }

    Ok(())
}

impl<T> Readable<Vec<T>> for Vec<T>
where
    T: Readable<T>,
{
    fn read(from: &mut dyn Buf, version: &mut Version) -> DifResult<Vec<T>> {
        read_vec::<T, T>(from, version, |_, _| false, |x| x)
    }
}

impl<T> Writable<Vec<T>> for Vec<T>
where
    T: Writable<T>,
{
    fn write(&self, to: &mut dyn BufMut, version: &Version) -> DifResult<()> {
        write_vec::<T, T>(&self, to, version)
    }
}

impl Readable<String> for String {
    fn read(from: &mut dyn Buf, version: &mut Version) -> DifResult<Self> {
        let length = u8::read(from, version)?;
        let bytes = from.take(length as usize).collect::<Vec<_>>();
        Ok(String::from_utf8(bytes).map_err(|e| DifError::from(e))?)
    }
}

impl Writable<String> for String {
    fn write(&self, to: &mut dyn BufMut, version: &Version) -> DifResult<()> {
        (self.len() as u8).write(to, version)?;
        for byte in self.bytes() {
            byte.write(to, version)?;
        }
        Ok(())
    }
}

macro_rules! primitive_readable {
    ($ty: ty, $read_fn: ident) => {
        impl Readable<$ty> for $ty {
            fn read(from: &mut dyn Buf, _version: &mut Version) -> DifResult<Self> {
                if from.remaining() < size_of::<Self>() {
                    return Err(DifError::from("EOF"));
                }
                Ok(from.$read_fn())
            }
        }
    };
}

macro_rules! primitive_writable {
    ($ty: ty, $write_fn: ident) => {
        impl Writable<$ty> for $ty {
            fn write(&self, to: &mut dyn BufMut, _version: &Version) -> DifResult<()> {
                Ok(to.$write_fn(*self))
            }
        }
    };
}

primitive_readable!(u8, get_u8);
primitive_readable!(u16, get_u16_le);
primitive_readable!(u32, get_u32_le);
primitive_readable!(u64, get_u64_le);

primitive_readable!(i8, get_i8);
primitive_readable!(i16, get_i16_le);
primitive_readable!(i32, get_i32_le);
primitive_readable!(i64, get_i64_le);

primitive_readable!(f32, get_f32_le);
primitive_readable!(f64, get_f64_le);

primitive_writable!(u8, put_u8);
primitive_writable!(u16, put_u16_le);
primitive_writable!(u32, put_u32_le);
primitive_writable!(u64, put_u64_le);

primitive_writable!(i8, put_i8);
primitive_writable!(i16, put_i16_le);
primitive_writable!(i32, put_i32_le);
primitive_writable!(i64, put_i64_le);

primitive_writable!(f32, put_f32_le);
primitive_writable!(f64, put_f64_le);

impl<T, X> Readable<TypedInt<T, X>> for TypedInt<T, X> where T: Readable<T>+Copy {
    fn read(from: &mut dyn Buf, version: &mut Version) -> DifResult<TypedInt<T, X>> {
        T::read(from, version).map(|b| Self::from(b))
    }
}

impl<T, X> Writable<TypedInt<T, X>> for TypedInt<T, X> where T: Writable<T>+Copy {
    fn write(&self, to: &mut dyn BufMut, version: &Version) -> DifResult<()> {
        self.inner().write(to, version)
    }
}
