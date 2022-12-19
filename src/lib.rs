#![deny(warnings)]

#![cfg_attr(not(test), no_std)]

#[doc=include_str!("../README.md")]
type _DocTestReadme = ();

use serde::{Serialize, Serializer};

pub trait SerializeSeed {
    type Value;

    fn serialize<S: Serializer>(&self, value: &Self::Value, serializer: S) -> Result<S::Ok, S::Error>;
}

pub struct ValueWithSeed<'a, Value, Seed>(pub &'a Value, pub Seed);

impl<'a, Value, Seed: SerializeSeed<Value=Value>> Serialize for ValueWithSeed<'a, Value, Seed> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.1.serialize(self.0, serializer)
    }
}
