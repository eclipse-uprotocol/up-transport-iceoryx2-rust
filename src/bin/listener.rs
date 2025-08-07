use async_trait::async_trait;
use env_logger;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Once;
use tokio::sync::Notify;
use up_rust::{UListener, UMessage, UTransport, UUri};
use up_transport_iceoryx2_rust::Iceoryx2Transport;

static INIT_LOGGER: Once = Once::new();

fn init_logger() {
    INIT_LOGGER.call_once(|| {
        env_logger::init();
    });
}

pub struct Receiver {
    notify: Arc<Notify>,
}

#[async_trait]
impl UListener for Receiver {
    async fn on_receive(&self, message: UMessage) {
        if let Some(payload) = &message.payload {
            print!("Received payload bytes (decimal): ");
            for byte in payload {
                print!("{} ", byte);
            }
            println!();
        } else {
            println!("Received message with no payload.");
        }
        self.notify.notify_one();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger();

    let notify = Arc::new(Notify::new());
    let receiver = Arc::new(Receiver {
        notify: notify.clone(),
    });

    let topic = UUri::from_str("//vehicle_shared/10A10B/1/CA5D")?;
    let transport = Iceoryx2Transport::new().unwrap();

    transport.register_listener(&topic, None, receiver).await?;

    println!("Listener started, waiting for messages...");

    loop {
        notify.notified().await;
        println!("Message received.");
    }
}
