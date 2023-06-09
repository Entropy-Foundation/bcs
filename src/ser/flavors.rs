//! Based on postcard flavors (https://github.com/jamesmunns/postcard/)

pub trait Flavor {
    /// The `Output` type is what this storage "resolves" to when the serialization is complete,
    /// such as a slice or a Vec of some sort.
    type Output;

    fn extend(&mut self, data: &[u8]);

    /// Finalize the serialization process
    fn finalize(self) -> Self::Output;
}

#[cfg(feature = "alloc")]
mod vec {
    use alloc::vec::Vec;

    use super::Flavor;

    impl Flavor for Vec<u8> {
        type Output = Self;

        fn extend(&mut self, data: &[u8]) {
            self.extend_from_slice(data)
        }

        fn finalize(self) -> Self::Output {
            self
        }
    }
}

#[derive(Default)]
pub struct Size(usize);

impl Flavor for Size {
    type Output = usize;

    fn extend(&mut self, b: &[u8]) {
        self.0 += b.len();
    }

    fn finalize(self) -> usize {
        self.0
    }
}
