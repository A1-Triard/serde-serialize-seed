#![deny(warnings)]

#![cfg_attr(not(test), no_std)]

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::DeserializeSeed;

    mod complex_type {
        use crate::SerializeSeed;
        use serde::{Deserializer, Serializer};
        use serde::de::{self, DeserializeSeed};
        use std::fmt::{self, Formatter};

        pub struct ComplexType(pub u8);

        pub struct ComplexTypeSerde {
            pub xor: u8,
        }

        struct ComplexTypeDeVisitor(ComplexTypeSerde);

        impl<'de> de::Visitor<'de> for ComplexTypeDeVisitor {
            type Value = ComplexType;

            fn expecting(&self, f: &mut Formatter) -> fmt::Result {
                write!(f, "u8")
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                let v = u8::try_from(v).map_err(|_| E::invalid_value(de::Unexpected::Unsigned(v), &self))?;
                Ok(ComplexType(v ^ self.0.xor))
            }
        }

        impl<'de> DeserializeSeed<'de> for ComplexTypeSerde {
            type Value = ComplexType;

            fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
                deserializer.deserialize_u8(ComplexTypeDeVisitor(self))
            }
        }

        impl SerializeSeed for ComplexTypeSerde {
            type Value = ComplexType;

            fn serialize<S: Serializer>(&self, value: &Self::Value, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_u8(value.0 ^ self.xor)
            }
        }
    }

    use complex_type::*;

    #[test]
    fn it_works() {
        let x = ComplexType(10);
        let json = serde_json::to_value(ValueWithSeed(&x, ComplexTypeSerde { xor: 0x34 })).unwrap();
        let y = ComplexTypeSerde { xor: 0x34 }.deserialize(json).unwrap();
        assert_eq!(x.0, y.0);
    }
}
