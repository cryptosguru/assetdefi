use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::rust::borrow::ToOwned;
use crate::rust::convert::TryFrom;
use crate::rust::vec;
use crate::rust::vec::Vec;
use crate::types::*;

/// Represents a vault id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Vid(pub H256, pub u32);

/// Represents an error when parsing Vid.
#[derive(Debug, Clone)]
pub enum ParseVidError {
    InvalidHex(hex::FromHexError),
    InvalidLength(usize),
}

impl Vid {
    pub fn to_vec(&self) -> Vec<u8> {
        let mut vec = Vec::with_capacity(36);
        vec.extend(self.0.as_ref());
        vec.extend(&self.1.to_le_bytes());
        vec
    }
}

impl TryFrom<&[u8]> for Vid {
    type Error = ParseVidError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != 36 {
            Err(ParseVidError::InvalidLength(slice.len()))
        } else {
            Ok(Self(
                H256(copy_u8_array(&slice[..32])),
                u32::from_le_bytes(copy_u8_array(&slice[32..])),
            ))
        }
    }
}

impl TypeId for Vid {
    #[inline]
    fn type_id() -> u8 {
        SCRYPTO_TYPE_VID
    }
}

impl Encode for Vid {
    fn encode_value(&self, encoder: &mut Encoder) {
        let bytes = self.to_vec();
        encoder.write_len(bytes.len());
        encoder.write_slice(&bytes);
    }
}

impl Decode for Vid {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let len = decoder.read_len()?;
        let slice = decoder.read_bytes(len)?;
        Self::try_from(slice).map_err(|_| DecodeError::InvalidCustomData(SCRYPTO_TYPE_VID))
    }
}

impl Describe for Vid {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_VID.to_owned(),
            generics: vec![],
        }
    }
}
