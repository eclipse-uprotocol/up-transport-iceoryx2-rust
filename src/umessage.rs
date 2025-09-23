// ################################################################################
// Copyright (c) 2025 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License Version 2.0 which is available at
// https: //www.apache.org/licenses/LICENSE-2.0
//
// SPDX-License-Identifier: Apache-2.0
// ################################################################################

use iceoryx2::prelude::ZeroCopySend;
use protobuf::Message;
use std::fmt::Debug;
use up_rust::UMessage;

const MAX_UMESSAGE_SIZE: usize = 4096;

/// Zero-copy safe wrapper for UMessage that serializes the protobuf data
/// into a fixed-size byte array to meet ZeroCopySend requirements.
///
/// See [ZeroCopySend's safety requirements](https://docs.rs/iceoryx2/latest/iceoryx2/prelude/trait.ZeroCopySend.html#safety) for more details.
///
/// ## DISCLAIMER
///
/// This code is a prototype to make UMessage work with iceoryx2's ZeroCopySend system
///
/// UMessage is not ZeroCopySend compatible as-is. If UMessages are sent
/// directly to an iceoryx2 publisher, it will compile. However, the
/// subscriber will receive a segmentation fault when receiving theUMessage
///
/// This essentially defeats the purpose of using iceoryx2 and
/// ZeroCopySend, as it copies the data into a fixed-size array and then out
/// of the array and back into a UMessage inside the `UTransport.send()` method
/// and the `UListener.on_receive()` method. The UTransport or UMessage
/// definition needs to be adjusted for this to truly be a zero-copy transport.
#[derive(Debug)]
#[repr(C)]
pub struct UMessageZeroCopy {
    data_length: u32,
    data: [u8; MAX_UMESSAGE_SIZE],
}

impl UMessageZeroCopy {
    /// Create a new UMessageZeroCopy from a UMessage
    pub fn new(message: UMessage) -> Result<Self, String> {
        let serialized_data = message
            .write_to_bytes()
            .map_err(|e| format!("Failed to serialize UMessage: {}", e))?;
        if serialized_data.len() > MAX_UMESSAGE_SIZE {
            return Err(format!(
                "UMessage too large: {} bytes (max: {})",
                serialized_data.len(),
                MAX_UMESSAGE_SIZE
            ));
        }
        let mut data = [0u8; MAX_UMESSAGE_SIZE];
        data[..serialized_data.len()].copy_from_slice(&serialized_data);
        Ok(Self {
            data_length: serialized_data.len() as u32,
            data,
        })
    }

    /// Convert back to UMessage
    pub fn to_umessage(&self) -> Result<UMessage, String> {
        let data_slice = &self.data[..self.data_length as usize];
        UMessage::parse_from_bytes(data_slice)
            .map_err(|e| format!("Failed to deserialize UMessage: {}", e))
    }

    /// Get the serialized data as a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        &self.data[..self.data_length as usize]
    }

    /// Get the length of the actual data
    pub fn len(&self) -> usize {
        self.data_length as usize
    }

    /// Check if the message is empty
    pub fn is_empty(&self) -> bool {
        self.data_length == 0
    }
}

unsafe impl ZeroCopySend for UMessageZeroCopy {
    unsafe fn type_name() -> &'static str {
        "UMessageZeroCopy"
    }
}

impl From<UMessage> for UMessageZeroCopy {
    fn from(message: UMessage) -> Self {
        Self::new(message).expect("Failed to convert UMessage to UMessageZeroCopy")
    }
}

impl TryFrom<UMessageZeroCopy> for UMessage {
    type Error = String;

    fn try_from(zero_copy: UMessageZeroCopy) -> Result<Self, Self::Error> {
        zero_copy.to_umessage()
    }
}
