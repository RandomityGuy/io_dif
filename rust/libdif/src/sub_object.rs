use crate::io::*;
use crate::types::*;
use bytes::{Buf, BufMut};

#[derive(Debug, Clone)]
pub struct SubObject {}

impl Readable<SubObject> for SubObject {
    fn read(_from: &mut dyn Buf, _version: &mut Version) -> DifResult<Self> {
        unimplemented!()
    }
}

impl Writable<SubObject> for SubObject {
    fn write(&self, _from: &mut dyn BufMut, _version: &Version) -> DifResult<()> {
        unimplemented!()
    }
}
