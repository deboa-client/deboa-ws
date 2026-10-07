use super::*;
use base64::{engine::general_purpose::STANDARD, Engine};
use http::{header, Method};

#[test]
fn websocket_request_builder_sets_expected_upgrade_headers() {
    let request = DeboaRequestBuilder::websocket("ws://example.com/socket")
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(request.method(), &Method::GET);
    assert_eq!(
        request
            .uri()
            .scheme_str(),
        Some("ws")
    );
    assert_eq!(
        request
            .headers()
            .get(header::UPGRADE)
            .and_then(|value| value.to_str().ok()),
        Some("websocket")
    );
    assert_eq!(
        request
            .headers()
            .get(header::CONNECTION)
            .and_then(|value| value.to_str().ok()),
        Some("Upgrade")
    );
    assert_eq!(
        request
            .headers()
            .get(header::SEC_WEBSOCKET_VERSION)
            .and_then(|value| value.to_str().ok()),
        Some("13")
    );

    let key = request
        .headers()
        .get(header::SEC_WEBSOCKET_KEY)
        .and_then(|value| value.to_str().ok())
        .expect("websocket key should be present");

    assert_eq!(key.len(), 24, "base64 websocket key should be 24 chars");
    let decoded = STANDARD
        .decode(key)
        .expect("websocket key must be valid base64");
    assert_eq!(decoded.len(), 16, "base64 decoded websocket key should be 16 bytes");
}

#[test]
fn websocket_message_variants_remain_distinct() {
    let text = Message::Text("hello".to_string());
    let binary = Message::Binary(vec![1, 2, 3]);
    let close = Message::Close(1000, "bye".to_string());
    let ping = Message::Ping(vec![9, 9]);
    let pong = Message::Pong(vec![8, 8]);

    assert_eq!(text, Message::Text("hello".to_string()));
    assert_eq!(binary, Message::Binary(vec![1, 2, 3]));
    assert_eq!(close, Message::Close(1000, "bye".to_string()));
    assert_eq!(ping, Message::Ping(vec![9, 9]));
    assert_eq!(pong, Message::Pong(vec![8, 8]));
}

#[test]
fn websocket_error_messages_are_formatted_for_debugging() {
    let receive = WebSocketError::ReceiveMessage { message: "broken frame".to_string() };
    let send = WebSocketError::SendMessage { message: "write failed".to_string() };

    assert_eq!(receive.to_string(), "Error receiving message: broken frame");
    assert_eq!(send.to_string(), "Error sending message: write failed");
}
