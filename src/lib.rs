use async_trait::async_trait;
use up_rust::{UMessage, UStatus, UTransport};

/// This will be the main struct for our uProtocol transport.
/// It will hold the state necessary to communicate with iceoryx2,
/// such as the service connection and active listeners.

pub struct Iceoryx2Transport {}

// The #[async_trait] attribute enables async functions in our trait impl.
#[async_trait]
impl UTransport for Iceoryx2Transport {
    /// The other trait methods (receive, register_listener, etc.) have
    /// default implementations, so we only need to provide `send` for now.
    async fn send(&self, _message: UMessage) -> Result<(), UStatus> {
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

}
