use super::*;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use log::info;
use tokio::sync::Notify;

use env_logger;
use iceoryx2::prelude::*;
use std::str::FromStr;
use up_rust::{
    MockUListener, UAttributes, UCode, UListener, UMessage, UMessageBuilder, UPayloadFormat,
    UStatus, UTransport, UUri,
};

use std::sync::Once;

static INIT_LOGGER: Once = Once::new();

fn init_logger() {
    INIT_LOGGER.call_once(|| {
        env_logger::init();
    });
}

const MESSAGE_DATA: &str = "Hello World!";

pub struct Receiver {
    expected: UMessage,
    notify: Arc<Notify>,
}

impl Receiver {
    pub fn new(expected: UMessage, notify: Arc<Notify>) -> Self {
        Self { expected, notify }
    }
}

#[async_trait]
impl UListener for Receiver {
    async fn on_receive(&self, message: UMessage) -> () {
        if let Some(payload) = &message.payload {
            println!("Received Message ID: {:#?}", message.id());
            if let (Some(expected_payload), Some(actual_payload)) =
                (&self.expected.payload, &message.payload)
            {
                assert_eq!(expected_payload, actual_payload);
            } else {
                panic!("Missing payloads in either expected or actual message");
            }
            self.notify.notify_one();
        }
    }
}

async fn register_listener_and_send(
    authority: &str,
    umessage: UMessage,
    source_filter: &UUri,
    sink_filter: Option<&UUri>,
) -> Result<(), Box<dyn std::error::Error>> {
    init_logger();
    let source_uri = UUri::try_from_parts(authority, 0xABC, 1, 0)?;
    let transport = Iceoryx2Transport::new().unwrap();
    let notify = Arc::new(Notify::new());
    let receiver = Arc::new(Receiver::new(umessage.clone(), notify.clone()));

    transport
        .register_listener(source_filter, sink_filter, receiver)
        .await
        .unwrap();

    // Send UMessage
    info!(
        "sending message: [id: {}, type: {}]",
        umessage.id_unchecked().to_hyphenated_string(),
        umessage.type_unchecked().to_cloudevent_type()
    );
    transport.send(umessage).await?;
    Ok(
        tokio::time::timeout(Duration::from_secs(3), notify.notified())
            .await
            .map_err(|_| {
                UStatus::fail_with_code(UCode::DEADLINE_EXCEEDED, "did not receive message in time")
            })?,
    )
}

#[test_case::test_case("vehicle1", 12_000, "//vehicle1/10A10B/1/CA5D", "//vehicle1/10A10B/1/CA5D"; "specific source filter")]
#[tokio::test(flavor = "multi_thread")]
async fn test_publish_gets_to_listener(
    authority: &str,
    ttl: u32,
    topic_uri: &str,
    source_filter_uri: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let topic = UUri::from_str(topic_uri)?;
    let source_filter = UUri::from_str(source_filter_uri)?;

    let payload = vec![1, 2, 3, 4, 5];

    let umessage = UMessageBuilder::publish(topic.clone())
        .with_priority(up_rust::UPriority::UPRIORITY_CS5)
        .with_traceparent("traceparent")
        .with_ttl(ttl)
        .build_with_payload(payload, UPayloadFormat::UPAYLOAD_FORMAT_RAW)?;

    register_listener_and_send(authority, umessage, &source_filter, None).await
}

#[tokio::test]
async fn test_exact_listener_dispatch() -> Result<(), Box<dyn std::error::Error>> {
    let transport = Iceoryx2Transport::new().unwrap();
    let listener_uri = UUri::from_str("//vehicle1/10A10B/1/CA5D")?;
    let matching_message = UMessageBuilder::publish(listener_uri.clone())
        .build_with_payload(vec![1, 2, 3], UPayloadFormat::UPAYLOAD_FORMAT_RAW)?;
    let non_matching_message =
        UMessageBuilder::publish(UUri::from_str("//vehicle2/10A10B/1/CA5D")?)
            .build_with_payload(vec![4, 5, 6], UPayloadFormat::UPAYLOAD_FORMAT_RAW)?;

    let received_notify = Arc::new(Notify::new());
    let listener = Arc::new(Receiver::new(
        matching_message.clone(),
        received_notify.clone(),
    ));

    transport
        .register_listener(&listener_uri, None, listener)
        .await?;
    transport.send(non_matching_message).await?;
    transport.send(matching_message).await?;

    tokio::time::timeout(Duration::from_secs(1), received_notify.notified()).await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_unregister_listener_stops_processing_of_messages() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    let transport = Iceoryx2Transport::new().unwrap();

    struct TestListener {
        hit_count: AtomicUsize,
    }

    #[async_trait::async_trait]
    impl UListener for TestListener {
        async fn on_receive(&self, _msg: UMessage) {
            let count = self.hit_count.fetch_add(1, Ordering::SeqCst);
            println!("Received a message, count = {}", count);
        }
    }

    let listener = Arc::new(TestListener {
        hit_count: AtomicUsize::new(0),
    });

    let uri = UUri::from_str(&format!("//vehicle{}/123/1/9000", std::process::id())).unwrap();
    let msg = UMessageBuilder::publish(uri.clone())
        .build()
        .expect("failed to build");

    // Registers listener
    transport
        .register_listener(&uri, None, listener.clone())
        .await
        .unwrap();

    // Let subscriber start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Send first message
    transport.send(msg.clone()).await.unwrap();

    // Wait for message processing
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify we received the first message
    assert!(
        listener.hit_count.load(Ordering::SeqCst) >= 1,
        "Listener should have received at least one message"
    );

    // Record count before unregister
    let count_before_unregister = listener.hit_count.load(Ordering::SeqCst);

    // Unregister
    transport
        .unregister_listener(&uri, None, listener.clone())
        .await
        .unwrap();

    // Wait for unregister to take effect
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Send more messages after unregister
    for _ in 0..3 {
        transport.send(msg.clone()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // Wait a bit more to ensure no messages are processed
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify no additional messages were received after unregister
    let count_after_unregister = listener.hit_count.load(Ordering::SeqCst);
    assert_eq!(
        count_before_unregister, count_after_unregister,
        "Listener should not receive messages after unregister. Before: {}, After: {}",
        count_before_unregister, count_after_unregister
    );
}

#[tokio::test]
async fn test_mock_listener() {
    use up_rust::MockUListener;
    let mut listener = MockUListener::new();

    listener.expect_on_receive().returning(|_message| {
        println!("Mock listener called!");
    });

    let message = UMessage::new();

    listener.on_receive(message).await;
}
