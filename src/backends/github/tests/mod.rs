use super::models::{PingEvent, PushEvent};

#[test]
fn test_parse_ping_event() {
    let event_str = include_str!("./ping_sample.json");
    let _: PingEvent = serde_json::from_str(event_str).expect("should deserialize");
}

#[test]
fn test_parse_push_event() {
    let event_str = include_str!("./push_sample.json");
    let _: PushEvent = serde_json::from_str(event_str).expect("should deserialize");
}
