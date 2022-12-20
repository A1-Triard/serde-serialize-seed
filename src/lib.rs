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
use phantom_type::PhantomType;
use serde::{Serialize, Serializer};
#[cfg(feature="alloc")]
use serde::{Deserialize, Deserializer};
#[cfg(feature="alloc")]
use serde::de::{self, DeserializeSeed, SeqAccess};
use serde::de::Error as de_Error;
use serde::ser::SerializeTuple;
#[cfg(feature="alloc")]
use serde::ser::SerializeSeq;

pub trait SerializeSeed {
    type Value: ?Sized;

    fn serialize<S: Serializer>(&self, value: &Self::Value, serializer: S) -> Result<S::Ok, S::Error>;
}

impl<T: SerializeSeed + ?Sized> SerializeSeed for &T {
    type Value = T::Value;

    fn serialize<S: Serializer>(&self, value: &Self::Value, serializer: S) -> Result<S::Ok, S::Error> {
        (*self).serialize(value, serializer)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ValueWithSeed<'a, Value: ?Sized, Seed>(pub &'a Value, pub Seed);

impl<'a, Value: ?Sized, Seed: SerializeSeed<Value=Value>> Serialize for ValueWithSeed<'a, Value, Seed> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.1.serialize(self.0, serializer)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StatelessSerde<T: ?Sized>(pub PhantomType<T>);

impl<T: Serialize + ?Sized> SerializeSeed for StatelessSerde<T> {
    type Value = T;

    fn serialize<S: Serializer>(&self, value: &Self::Value, serializer: S) -> Result<S::Ok, S::Error> {
        value.serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de>> DeserializeSeed<'de> for StatelessSerde<T> {
    type Value = T;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        T::deserialize(deserializer)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PairSerde<U, V>(pub U, pub V);

impl<U: SerializeSeed, V: SerializeSeed> SerializeSeed for PairSerde<U, V> where U::Value: Sized, V::Value: Sized {
    type Value = (U::Value, V::Value);

    fn serialize<S: Serializer>(&self, value: &Self::Value, serializer: S) -> Result<S::Ok, S::Error> {
        let mut serializer = serializer.serialize_tuple(2)?;
        serializer.serialize_element(&ValueWithSeed(&value.0, &self.0))?;
        serializer.serialize_element(&ValueWithSeed(&value.1, &self.1))?;
        serializer.end()
    }
}

struct PairDeVisitor<U, V>(PairSerde<U, V>);

impl<'de, U: DeserializeSeed<'de>, V: DeserializeSeed<'de>> de::Visitor<'de> for PairDeVisitor<U, V> where
    U::Value: Sized, V::Value: Sized
{
    type Value = (U::Value, V::Value);

    fn expecting(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "pair")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
        let u = seq.next_element_seed(self.0.0)?
            .ok_or_else(|| A::Error::invalid_length(0, &"pair"))?;
        let v = seq.next_element_seed(self.0.1)?
            .ok_or_else(|| A::Error::invalid_length(1, &"pair"))?;
        Ok((u, v))
    }
}

impl<'de, U: DeserializeSeed<'de>, V: DeserializeSeed<'de>> DeserializeSeed<'de> for PairSerde<U, V> where
    U::Value: Sized, V::Value: Sized
{
    type Value = (U::Value, V::Value);

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_tuple(2, PairDeVisitor(self))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Tuple4Serde<T1, T2, T3, T4>(pub T1, pub T2, pub T3, pub T4);

impl<
    T1: SerializeSeed,
    T2: SerializeSeed,
    T3: SerializeSeed,
    T4: SerializeSeed
> SerializeSeed for Tuple4Serde<T1, T2, T3, T4> where
    T1::Value: Sized, T2::Value: Sized, T3::Value: Sized, T4::Value: Sized
{
    type Value = (T1::Value, T2::Value, T3::Value, T4::Value);

    fn serialize<S: Serializer>(&self, value: &Self::Value, serializer: S) -> Result<S::Ok, S::Error> {
        let mut serializer = serializer.serialize_tuple(4)?;
        serializer.serialize_element(&ValueWithSeed(&value.0, &self.0))?;
        serializer.serialize_element(&ValueWithSeed(&value.1, &self.1))?;
        serializer.serialize_element(&ValueWithSeed(&value.2, &self.2))?;
        serializer.serialize_element(&ValueWithSeed(&value.3, &self.3))?;
        serializer.end()
    }
}

struct Tuple4DeVisitor<T1, T2, T3, T4>(Tuple4Serde<T1, T2, T3, T4>);

impl<
    'de,
    T1: DeserializeSeed<'de>,
    T2: DeserializeSeed<'de>,
    T3: DeserializeSeed<'de>,
    T4: DeserializeSeed<'de>
> de::Visitor<'de> for Tuple4DeVisitor<T1, T2, T3, T4> where
    T1::Value: Sized, T2::Value: Sized, T3::Value: Sized, T4::Value: Sized
{
    type Value = (T1::Value, T2::Value, T3::Value, T4::Value);

    fn expecting(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "tuple 4")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
        let t1 = seq.next_element_seed(self.0.0)?
            .ok_or_else(|| A::Error::invalid_length(0, &"tuple 4"))?;
        let t2 = seq.next_element_seed(self.0.1)?
            .ok_or_else(|| A::Error::invalid_length(1, &"tuple 4"))?;
        let t3 = seq.next_element_seed(self.0.2)?
            .ok_or_else(|| A::Error::invalid_length(2, &"tuple 4"))?;
        let t4 = seq.next_element_seed(self.0.3)?
            .ok_or_else(|| A::Error::invalid_length(3, &"tuple 4"))?;
        Ok((t1, t2, t3, t4))
    }
}

impl<
    'de,
    T1: DeserializeSeed<'de>,
    T2: DeserializeSeed<'de>,
    T3: DeserializeSeed<'de>,
    T4: DeserializeSeed<'de>
> DeserializeSeed<'de> for Tuple4Serde<T1, T2, T3, T4> where
    T1::Value: Sized, T2::Value: Sized, T3::Value: Sized, T4::Value: Sized
{
    type Value = (T1::Value, T2::Value, T3::Value, T4::Value);

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_tuple(4, Tuple4DeVisitor(self))
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
