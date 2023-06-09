use serde::{ser, Serialize};

use crate::{Error, Result, MAX_CONTAINER_DEPTH};

use self::{flavors::Flavor, serializer::Serializer};

pub mod flavors;
mod serializer;

pub fn serialize_with_flavor<T, S, O>(value: &T, mut storage: S) -> Result<O>
where
    T: Serialize + ?Sized,
    S: Flavor<Output = O>,
{
    let serializer = Serializer::new(&mut storage, MAX_CONTAINER_DEPTH);
    value.serialize(serializer)?;
    Ok(storage.finalize())
}

pub fn serialize_with_flavor_and_limit<T, S, O>(
    value: &T,
    mut storage: S,
    limit: usize,
) -> Result<O>
where
    T: Serialize + ?Sized,
    S: Flavor<Output = O>,
{
    if limit > MAX_CONTAINER_DEPTH {
        return Err(Error::NotSupported("limit exceeds the max allowed depth"));
    }

    let serializer = Serializer::new(&mut storage, limit);
    value.serialize(serializer)?;
    Ok(storage.finalize())
}

pub fn is_human_readable() -> bool {
    let mut output = Vec::new();
    let serializer = Serializer::new(&mut output, crate::MAX_CONTAINER_DEPTH);
    ser::Serializer::is_human_readable(&serializer)
}

/// Same as `to_bytes` but only return the size of the serialized bytes.
pub fn serialized_size<T>(value: &T) -> Result<usize>
where
    T: ?Sized + Serialize,
{
    let counter = flavors::Size::default();
    serialize_with_flavor(value, counter)
}

/// Serialize the given data structure as a `Vec<u8>` of BCS.
///
/// Serialization can fail if `T`'s implementation of `Serialize` decides to
/// fail, if `T` contains sequences which are longer than `MAX_SEQUENCE_LENGTH`,
/// or if `T` attempts to serialize an unsupported datatype such as a f32,
/// f64, or char.
///
/// # Examples
///
/// ```
/// use bcs::to_bytes;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Ip([u8; 4]);
///
/// #[derive(Serialize)]
/// struct Port(u16);
///
/// #[derive(Serialize)]
/// struct Service {
///     ip: Ip,
///     port: Vec<Port>,
///     connection_max: Option<u32>,
///     enabled: bool,
/// }
///
/// let service = Service {
///     ip: Ip([192, 168, 1, 1]),
///     port: vec![Port(8001), Port(8002), Port(8003)],
///     connection_max: Some(5000),
///     enabled: false,
/// };
///
/// let bytes = to_bytes(&service).unwrap();
/// let expected = vec![
///     0xc0, 0xa8, 0x01, 0x01, 0x03, 0x41, 0x1f, 0x42,
///     0x1f, 0x43, 0x1f, 0x01, 0x88, 0x13, 0x00, 0x00,
///     0x00,
/// ];
/// assert_eq!(bytes, expected);
/// ```
#[cfg(feature = "alloc")]
pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>>
where
    T: ?Sized + Serialize,
{
    let output = Vec::new();
    serialize_with_flavor(value, output)
}

/// Same as `to_bytes` but use `limit` as max container depth instead of MAX_CONTAINER_DEPTH
/// Note that `limit` has to be lower than MAX_CONTAINER_DEPTH
#[cfg(feature = "alloc")]
pub fn to_bytes_with_limit<T>(value: &T, limit: usize) -> Result<Vec<u8>>
where
    T: ?Sized + Serialize,
{
    if limit > MAX_CONTAINER_DEPTH {
        return Err(Error::NotSupported("limit exceeds the max allowed depth"));
    }

    let output = Vec::new();
    serialize_with_flavor_and_limit(value, output, limit)
}
