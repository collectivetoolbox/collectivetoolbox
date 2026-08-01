use zeroize::ZeroizeOnDrop;

/// Simple secret holder that zeroes memory on drop.
#[derive(Debug, Default, Clone, PartialEq, ZeroizeOnDrop)]
pub(crate) struct Secret {
    bytes: Vec<u8>,
}
impl Secret {
    pub(crate) fn new(bytes: Vec<u8>) -> Self {
        Secret { bytes }
    }
    pub(crate) fn as_slice(&self) -> &[u8] {
        &self.bytes
    }
}
