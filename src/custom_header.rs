use iceoryx2::prelude::*;
use up_rust::{UAttributes, UMessage, UStatus, UCode};
use iceoryx2_bb_container::vec::FixedSizeVec;
use protobuf::Message; 

const MAX_FEASIBLE_UATTRIBUTES_SERIALIZED_LENGTH: usize = 1000;
const UPROTOCOL_MAJOR_VERSION: u8 = 1;

#[derive(Default, Debug, ZeroCopySend)]
#[type_name("CustomHeader")]
#[repr(C)]
pub struct CustomHeader {
    uprotocol_major_version: u8,
    uattributes_serialized: FixedSizeVec<u8, MAX_FEASIBLE_UATTRIBUTES_SERIALIZED_LENGTH>
}

impl CustomHeader {
    pub fn from_user_header(header: &Self) -> Result<Self, UStatus> {
        Ok(Self {
            uprotocol_major_version: header.uprotocol_major_version,
            uattributes_serialized: header.uattributes_serialized.clone(),
        })
    }

    pub fn from_message(message: &UMessage) -> Result<Self, UStatus> {
        let uattributes = message
            .attributes
            .as_ref()
            .ok_or_else(|| UStatus::fail_with_code(UCode::INVALID_ARGUMENT, "Missing attributes"))?;

        let serialized = uattributes
            .write_to_bytes()
            .map_err(|_| UStatus::fail_with_code(UCode::INTERNAL, "Failed to serialize attributes"))?;

        if serialized.len() > MAX_FEASIBLE_UATTRIBUTES_SERIALIZED_LENGTH {
            return Err(UStatus::fail_with_code(
                UCode::RESOURCE_EXHAUSTED,
                "Serialized UAttributes too large for header",
            ));
        }

        let mut vec = FixedSizeVec::new();
        if !vec.extend_from_slice(&serialized) {
            return Err(UStatus::fail_with_code(
                UCode::RESOURCE_EXHAUSTED,
                "Failed to insert serialized data into FixedSizeVec",
            ));
        }

        Ok(Self {
            uprotocol_major_version: UPROTOCOL_MAJOR_VERSION,
            uattributes_serialized: vec,
        })
    }
}

impl TryFrom<&CustomHeader> for UAttributes {
    type Error = UStatus;

    fn try_from(header: &CustomHeader) -> Result<Self, Self::Error> {
        let bytes = header.uattributes_serialized.as_slice();
        protobuf::Message::parse_from_bytes(bytes)
            .map_err(|_| UStatus::fail_with_code(UCode::INTERNAL, "Failed to parse UAttributes"))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use up_rust::{UMessageType, UPriority};
    use protobuf::MessageField;

    fn dummy_uattrs() -> UAttributes {
        let mut attrs = UAttributes::default();
        attrs.type_ = UMessageType::UMESSAGE_TYPE_PUBLISH.into();
        attrs.priority = UPriority::UPRIORITY_CS4.into();
        attrs
    }

    #[test]
    fn test_from_user_header_clones_data() {
        let mut vec = FixedSizeVec::new();
        vec.extend_from_slice(&[1, 2, 3]);

        let original = CustomHeader {
            uprotocol_major_version: 2,
            uattributes_serialized: vec,
        };
        let cloned = CustomHeader::from_user_header(&original).unwrap();
        assert_eq!(cloned.uprotocol_major_version, 2);
        assert_eq!(cloned.uattributes_serialized.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn test_from_message_success() {
        let attrs = dummy_uattrs();
        let msg = UMessage {
            attributes: MessageField::some(attrs.clone()),
            ..Default::default()
        };

        let header = CustomHeader::from_message(&msg).unwrap();
        let deserialized: UAttributes = (&header).try_into().unwrap(); 

        assert_eq!(deserialized.type_, attrs.type_);
        assert_eq!(deserialized.priority, attrs.priority);
    }

    #[test]
    fn test_from_message_fails_on_none() {
        let msg = UMessage {
            attributes: MessageField::none(),
            ..Default::default()
        };
        let err = CustomHeader::from_message(&msg).unwrap_err();
        assert_eq!(err.code, UCode::INVALID_ARGUMENT.into());
    }

    #[test]
    fn test_into_uattributes_from_header_roundtrip() {
        let original = dummy_uattrs();
        let msg = UMessage {
            attributes: MessageField::some(original.clone()),
            ..Default::default()
        };
        let header = CustomHeader::from_message(&msg).unwrap();
        let deserialized: UAttributes = (&header).try_into().unwrap(); 

        assert_eq!(deserialized.type_, original.type_);
        assert_eq!(deserialized.priority, original.priority);
    }
}
