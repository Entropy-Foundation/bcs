use crate::{Error, Result};

use super::flavors::Flavor;
use serde::{ser, Serialize};

/// Serialization implementation for BCS
pub struct Serializer<'a, F> {
    output: &'a mut F,
    max_remaining_depth: usize,
}

impl<'a, F> Serializer<'a, F>
where
    F: Flavor,
{
    /// Creates a new `Serializer` which will emit BCS.
    pub fn new(output: &'a mut F, max_remaining_depth: usize) -> Self {
        Self {
            output,
            max_remaining_depth,
        }
    }

    fn output_u32_as_uleb128(&mut self, mut value: u32) -> Result<()> {
        while value >= 0x80 {
            // Write 7 (lowest) bits of data and set the 8th bit to 1.
            let byte = (value & 0x7f) as u8;
            self.output.extend(&[byte | 0x80]);
            value >>= 7;
        }
        // Write the remaining bits of data and set the highest bit to 0.
        self.output.extend(&[value as u8]);
        Ok(())
    }

    fn output_variant_index(&mut self, v: u32) -> Result<()> {
        self.output_u32_as_uleb128(v)
    }

    /// Serialize a sequence length as a u32.
    fn output_seq_len(&mut self, len: usize) -> Result<()> {
        if len > crate::MAX_SEQUENCE_LENGTH {
            return Err(Error::ExceededMaxLen(len));
        }
        self.output_u32_as_uleb128(len as u32)
    }

    fn enter_named_container(&mut self, name: &'static str) -> Result<()> {
        if self.max_remaining_depth == 0 {
            return Err(Error::ExceededContainerDepthLimit(name));
        }
        self.max_remaining_depth -= 1;
        Ok(())
    }
}

impl<'a, F> ser::Serializer for Serializer<'a, F>
where
    F: Flavor,
{
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    #[cfg(feature = "alloc")]
    type SerializeMap = map_ser::MapSerializer<'a, F>;
    #[cfg(not(feature = "alloc"))]
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.serialize_u8(v.into())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_u8(v as u8)
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_u16(v as u16)
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_u32(v as u32)
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.serialize_u64(v as u64)
    }

    fn serialize_i128(self, v: i128) -> Result<()> {
        self.serialize_u128(v as u128)
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.output.extend(&[v]);
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.output.extend(&v.to_le_bytes());
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.output.extend(&v.to_le_bytes());
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.output.extend(&v.to_le_bytes());
        Ok(())
    }

    fn serialize_u128(self, v: u128) -> Result<()> {
        self.output.extend(&v.to_le_bytes());
        Ok(())
    }

    fn serialize_f32(self, _v: f32) -> Result<()> {
        Err(Error::NotSupported("serialize_f32"))
    }

    fn serialize_f64(self, _v: f64) -> Result<()> {
        Err(Error::NotSupported("serialize_f64"))
    }

    fn serialize_char(self, _v: char) -> Result<()> {
        Err(Error::NotSupported("serialize_char"))
    }

    // Just serialize the string as a raw byte array
    fn serialize_str(self, v: &str) -> Result<()> {
        self.serialize_bytes(v.as_bytes())
    }

    // Serialize a byte array as an array of bytes.
    fn serialize_bytes(mut self, v: &[u8]) -> Result<()> {
        self.output_seq_len(v.len())?;
        self.output.extend(v);
        Ok(())
    }

    // An absent optional is represented as `00`
    fn serialize_none(self) -> Result<()> {
        self.serialize_u8(0)
    }

    // A present optional is represented as `01` followed by the serialized value
    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.output.extend(&[1]);
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        Ok(())
    }

    fn serialize_unit_struct(mut self, name: &'static str) -> Result<()> {
        self.enter_named_container(name)?;
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        mut self,
        name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        self.enter_named_container(name)?;
        self.output_variant_index(variant_index)
    }

    fn serialize_newtype_struct<T>(mut self, name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.enter_named_container(name)?;
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        mut self,
        name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.enter_named_container(name)?;
        self.output_variant_index(variant_index)?;
        value.serialize(self)
    }

    // The start of the sequence, each value, and the end are three separate
    // method calls. This one is responsible only for serializing the start,
    // which for BCS is either nothing for fixed structures or for variable
    // length structures, the length encoded as a u32.
    fn serialize_seq(mut self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        if let Some(len) = len {
            self.output_seq_len(len)?;
            Ok(self)
        } else {
            Err(Error::MissingLen)
        }
    }

    // Tuples are fixed sized structs so we don't need to encode the length
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        mut self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.enter_named_container(name)?;
        Ok(self)
    }

    fn serialize_tuple_variant(
        mut self,
        name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.enter_named_container(name)?;
        self.output_variant_index(variant_index)?;
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        #[cfg(feature = "alloc")]
        {
            Ok(map_ser::MapSerializer::new(self))
        }
        #[cfg(not(feature = "alloc"))]
        {
            Ok(self)
        }
    }

    fn serialize_struct(
        mut self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct> {
        self.enter_named_container(name)?;
        Ok(self)
    }

    fn serialize_struct_variant(
        mut self,
        name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.enter_named_container(name)?;
        self.output_variant_index(variant_index)?;
        Ok(self)
    }

    // BCS is not a human readable format
    fn is_human_readable(&self) -> bool {
        false
    }
}

impl<'a, F> ser::SerializeSeq for Serializer<'a, F>
where
    F: Flavor,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(Serializer::new(self.output, self.max_remaining_depth))
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, F> ser::SerializeTuple for Serializer<'a, F>
where
    F: Flavor,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(Serializer::new(self.output, self.max_remaining_depth))
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, F> ser::SerializeTupleStruct for Serializer<'a, F>
where
    F: Flavor,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(Serializer::new(self.output, self.max_remaining_depth))
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, F> ser::SerializeTupleVariant for Serializer<'a, F>
where
    F: Flavor,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(Serializer::new(self.output, self.max_remaining_depth))
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

#[doc(hidden)]
#[cfg(feature = "alloc")]
mod map_ser {
    use super::Serializer;
    use crate::ser::flavors::Flavor;
    use crate::{error::Error, Result};
    use alloc::vec::Vec;
    use serde::{ser, Serialize};

    pub struct MapSerializer<'a, F> {
        serializer: Serializer<'a, F>,
        entries: Vec<(Vec<u8>, Vec<u8>)>,
        next_key: Option<Vec<u8>>,
    }

    impl<'a, F> MapSerializer<'a, F> {
        pub fn new(serializer: Serializer<'a, F>) -> Self {
            MapSerializer {
                serializer,
                entries: Vec::new(),
                next_key: None,
            }
        }
    }

    impl<'a, F> ser::SerializeMap for MapSerializer<'a, F>
    where
        F: Flavor,
    {
        type Ok = ();
        type Error = Error;

        fn serialize_key<T>(&mut self, key: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
        {
            if self.next_key.is_some() {
                return Err(Error::ExpectedMapValue);
            }

            let mut output = Vec::new();
            key.serialize(Serializer::new(
                &mut output,
                self.serializer.max_remaining_depth,
            ))?;
            self.next_key = Some(output);
            Ok(())
        }

        fn serialize_value<T>(&mut self, value: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
        {
            match self.next_key.take() {
                Some(key) => {
                    let mut output = Vec::new();
                    value.serialize(Serializer::new(
                        &mut output,
                        self.serializer.max_remaining_depth,
                    ))?;
                    self.entries.push((key, output));
                    Ok(())
                }
                None => Err(Error::ExpectedMapKey),
            }
        }

        fn end(mut self) -> Result<()> {
            if self.next_key.is_some() {
                return Err(Error::ExpectedMapValue);
            }
            self.entries.sort_by(|e1, e2| e1.0.cmp(&e2.0));
            self.entries.dedup_by(|e1, e2| e1.0.eq(&e2.0));

            let len = self.entries.len();
            self.serializer.output_seq_len(len)?;

            for (key, value) in &self.entries {
                self.serializer.output.extend(key);
                self.serializer.output.extend(value);
            }

            Ok(())
        }
    }
}

impl<'a, F> ser::SerializeMap for Serializer<'a, F>
where
    F: Flavor,
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, _key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::NotSupported(
            "maps are not supported for non alloc environment",
        ))
    }

    fn serialize_value<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::NotSupported(
            "maps are not supported for non alloc environment",
        ))
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, F> ser::SerializeStruct for Serializer<'a, F>
where
    F: Flavor,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(Serializer::new(self.output, self.max_remaining_depth))
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, F> ser::SerializeStructVariant for Serializer<'a, F>
where
    F: Flavor,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(Serializer::new(self.output, self.max_remaining_depth))
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}
