use env_logger;
use std::str::FromStr;
use std::sync::Once;

use up_rust::{UMessageBuilder, UPayloadFormat, UTransport, UUri};
use up_transport_iceoryx2_rust::Iceoryx2Transport;

static INIT_LOGGER: Once = Once::new();

fn init_logger() {
    INIT_LOGGER.call_once(|| {
        env_logger::init();
    });
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger();

    let topic = UUri::from_str("//vehicle_shared/10A10B/1/CA5D")?;
    let transport = Iceoryx2Transport::new().unwrap();

    let mut payload: Vec<u8> = vec![1, 2, 3, 4, 5, 6];

    loop {
        // increment each byte
        for b in payload.iter_mut() {
            *b = b.wrapping_add(1);
        }

        let umessage = UMessageBuilder::publish(topic.clone())
            .build_with_payload(payload.clone(), UPayloadFormat::UPAYLOAD_FORMAT_RAW)?;

        transport.send(umessage).await?;

        println!("Message sent. Payload: {:?}", payload);

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
