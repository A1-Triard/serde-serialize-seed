#![deny(warnings)]

#![cfg_attr(not(test), no_std)]

#[cfg(test)]
extern crate core;

#[cfg(feature="alloc")]
extern crate alloc;

#[doc=include_str!("../README.md")]
type _DocTestReadme = ();

#[cfg(feature="alloc")]
use alloc::vec::Vec;
#[cfg(feature="alloc")]
use core::fmt::{self, Formatter};
use serde::{Serialize, Serializer};
#[cfg(feature="alloc")]
use serde::{Deserializer};
#[cfg(feature="alloc")]
use serde::de::{self, DeserializeSeed, SeqAccess};
#[cfg(feature="alloc")]
use serde::ser::SerializeSeq;

pub trait SerializeSeed {
    type Value: ?Sized;

    fn serialize<S: Serializer>(&self, value: &Self::Value, serializer: S) -> Result<S::Ok, S::Error>;
}

#[derive(Debug, Clone, Copy)]
pub struct ValueWithSeed<'a, Value: ?Sized, Seed>(pub &'a Value, pub Seed);

impl<'a, Value: ?Sized, Seed: SerializeSeed<Value=Value>> Serialize for ValueWithSeed<'a, Value, Seed> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.1.serialize(self.0, serializer)
    }
}

#[cfg(feature="alloc")]
#[derive(Debug, Clone, Copy)]
pub struct VecSerde<T>(pub T);

#[cfg(feature="alloc")]
impl<T: SerializeSeed + Clone> SerializeSeed for VecSerde<T> where T::Value: Sized {
    type Value = [T::Value];

    fn serialize<S: Serializer>(&self, value: &Self::Value, serializer: S) -> Result<S::Ok, S::Error> {
        let mut serializer = serializer.serialize_seq(Some(value.len()))?;
        for item in value {
            serializer.serialize_element(&ValueWithSeed(item, self.0.clone()))?;
        }
        serializer.end()
    }
}

#[cfg(feature="alloc")]
struct VecDeVisitor<T>(VecSerde<T>);

#[cfg(feature="alloc")]
impl<'de, T: DeserializeSeed<'de> + Clone> de::Visitor<'de> for VecDeVisitor<T> where T::Value: Sized {
    type Value = Vec<T::Value>;

    fn expecting(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "vector")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
        let mut vec = seq.size_hint().map_or_else(Vec::new, Vec::with_capacity);
        while let Some(f) = seq.next_element_seed(self.0.0.clone())? {
            vec.push(f);
        }
        Ok(vec)
    }
}

#[cfg(feature="alloc")]
impl<'de, T: DeserializeSeed<'de> + Clone> DeserializeSeed<'de> for VecSerde<T> where T::Value: Sized {
    type Value = Vec<T::Value>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_seq(VecDeVisitor(self))
    }
}
