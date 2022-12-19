![maintenance: actively developed](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)

# `serde-serialize-seed`

The `SerializeSeed` trait for convinience.

## Example

```rust
mod complex_type {
    use serde::{Deserializer, Serializer};
    use serde::de::{self, DeserializeSeed};
    use serde_serialize_seed::SerializeSeed;
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
use serde::de::DeserializeSeed;

fn main() {
    let x = ComplexType(10);
    let json = serde_json::to_value(ValueWithSeed(&x, ComplexTypeSerde { xor: 0x34 })).unwrap();
    let y = ComplexTypeSerde { xor: 0x34 }.deserialize(json).unwrap();
    assert_eq!(x.0, y.0);
}
```
