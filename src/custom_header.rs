use iceoryx2::prelude::*;
use up_rust::{UAttributes, UMessage, UStatus};

#[derive(Default, Debug, ZeroCopySend)]
#[type_name("CustomHeader")]
#[repr(C)]
pub struct CustomHeader {
    pub version: i32,
    pub timestamp: u64,
}

impl CustomHeader {
    pub fn from_user_header(header: &Self) -> Result<Self, UStatus> {
        Ok(Self {
            version: header.version,
            timestamp: header.timestamp,
        })
    }

    pub fn from_message(message: &UMessage) -> Result<Self, UStatus> {
        // Extract basic information from UMessage attributes
        let version = if let Some(_attributes) = &message.attributes.0 {
            1 // Use a default version for now
        } else {
            1 // Default version
        };

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Ok(Self { version, timestamp })
    }
}

// Assuming UAttributes has a field called `fields` which is Vec<(String, String)>
// Adjust if your actual UAttributes is different
impl From<&CustomHeader> for UAttributes {
    fn from(header: &CustomHeader) -> UAttributes {
        let mut attrs = UAttributes::default();

        // Map CustomHeader fields back to UAttributes using available fields
        // Based on the error message, available fields are: id, type_, source, sink, priority, etc.

        // Set default values for required fields
        attrs.type_ = up_rust::UMessageType::UMESSAGE_TYPE_PUBLISH.into();
        attrs.priority = up_rust::UPriority::UPRIORITY_CS4.into();

        // Note: version and timestamp information is preserved in the CustomHeader
        // but UAttributes doesn't have direct fields for them, so we use defaults

        attrs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_custom_header_from_user_header() {
        let header = CustomHeader {
            version: 2,
            timestamp: 1000,
        };
        let new_header = CustomHeader::from_user_header(&header).unwrap();
        assert_eq!(new_header.version, 2);
        assert_eq!(new_header.timestamp, 1000);
    }
}
