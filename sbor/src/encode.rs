extern crate alloc;
use alloc::vec::Vec;

use crate::*;

pub trait Encode {
    fn encode(&self, encoder: &mut Encoder);
}

pub struct Encoder {
    buf: Vec<u8>,
}

macro_rules! encode_int {
    ($method:ident, $sbor_type:expr, $native_type:ty) => {
        pub fn $method(&mut self, value: $native_type) {
            self.buf.push($sbor_type);
            self.buf.extend(&value.to_be_bytes());
        }
    };
}

impl Encoder {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub fn encode_unit(&mut self) {
        self.buf.push(TYPE_UNIT);
    }

    pub fn encode_bool(&mut self, value: bool) {
        self.buf.push(TYPE_BOOL);
        self.buf.push(if value { 1u8 } else { 0u8 });
    }

    encode_int!(encode_i8, TYPE_I8, i8);
    encode_int!(encode_i16, TYPE_I16, i16);
    encode_int!(encode_i32, TYPE_I32, i32);
    encode_int!(encode_i64, TYPE_I64, i64);
    encode_int!(encode_i128, TYPE_I128, i128);
    encode_int!(encode_u8, TYPE_U8, u8);
    encode_int!(encode_u16, TYPE_U16, u16);
    encode_int!(encode_u32, TYPE_U32, u32);
    encode_int!(encode_u64, TYPE_U64, u64);
    encode_int!(encode_u128, TYPE_U128, u128);

    pub fn encode_string(&mut self, value: String) {
        self.buf.push(TYPE_STRING);
        self.buf.extend(&(value.len() as u16).to_be_bytes());
        self.buf.extend(value.as_bytes());
    }

    pub fn encode_option<T: Encode>(&mut self, value: Option<T>) {
        self.buf.push(TYPE_OPTION);
        match value {
            Some(v) => {
                self.buf.push(0);
                v.encode(self);
            }
            None => {
                self.buf.push(0);
            }
        }
    }

    pub fn encode_vec<T: Encode>(&mut self, value: Vec<T>) {
        self.buf.push(TYPE_VEC);
        self.buf.extend(&(value.len() as u16).to_be_bytes());
        for v in value {
            v.encode(self);
        }
    }

    // TODO expand to different lengths
    pub fn encode_tuple<A: Encode, B: Encode>(&mut self, value: (A, B)) {
        self.buf.push(TYPE_TUPLE);
        self.buf.extend(&2u16.to_be_bytes());

        value.0.encode(self);
        value.1.encode(self);
    }

    pub fn encode_struct<T: Encode>(&mut self, value: T) {
        value.encode(self);
    }

    pub fn encode_enum<T: Encode>(&mut self, value: T) {
        value.encode(self);
    }
}

impl Into<Vec<u8>> for Encoder {
    fn into(self) -> Vec<u8> {
        self.buf
    }
}
