#![feature(start)]

#![deny(warnings)]

#![no_std]

#[cfg(windows)]
#[link(name="msvcrt")]
extern { }

mod no_std {
    use core::panic::PanicInfo;
    use exit_no_std::exit;

    #[panic_handler]
    fn panic(_info: &PanicInfo) -> ! {
        exit(99)
    }

    #[cfg(windows)]
    #[no_mangle]
    fn rust_oom(_layout: core::alloc::Layout) -> ! {
        exit(98)
    }
}

use core::fmt::{self, Formatter};
use serde::{Deserializer, Serializer};
use serde::de::{self, DeserializeSeed};
use serde_serialize_seed::SerializeSeed;

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

#[start]
pub fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let _ = ComplexType(0);
    0
}
