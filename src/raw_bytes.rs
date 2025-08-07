use iceoryx2::prelude::*;
use iceoryx2_bb_container::{byte_string::FixedSizeByteString, vec::FixedSizeVec};
/// A minimal wrapper to hold raw byte payloads for Iceoryx2.
///
/// Iceoryx2 requires payload types to implement ZeroCopySend.
/// This struct makes raw Vec<u8> compatible.
const MAX_RAW_BYTES_SIZE: usize = 4096;
#[derive(Debug, Clone, PartialEq, Default)]
#[repr(C)]
pub struct RawBytes {
    pub data: FixedSizeVec<u8, MAX_RAW_BYTES_SIZE>,
}

unsafe impl ZeroCopySend for RawBytes {}

impl RawBytes {
    /// Converts this RawBytes into a Vec<u8>.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.data.as_slice().to_vec()
    }

    /// Creates a RawBytes struct from a Vec<u8>.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut data = FixedSizeVec::new();
        for &byte in bytes {
            let _ok = data.push(byte);
        }
        RawBytes { data }
    }
}
