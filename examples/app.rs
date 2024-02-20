fn on_message(kind: u32, buf: Vec<u8>) {
    if kind == 42 {
        let received_message = std::str::from_utf8(&buf).unwrap();
        println!("Received message: kind={}, buf={}", kind, received_message);
    } else if kind == 43 {
        let received_message = std::str::from_utf8(&buf).unwrap();
        let breadcrumb: sentry::protocol::Breadcrumb =
            serde_json::from_str(received_message).unwrap();
        println!("Received breadcrumb: kind={}, buf={:?}", kind, breadcrumb);
    } else {
        println!("Received unknown: kind={}, buf={:?}", kind, buf);
    }
}

fn main() {
    let client = sentry::init("http://abc123@127.0.0.1:8123/12345");

    // Everything before here runs in both app and crash reporter processes
    let guard = sentry_rust_minidump::init_w_message_support(&client, Some(on_message));
    // Everything after here runs in only the app process

    assert!(guard.is_ok());

    let client_handle = guard.unwrap();
    let sent_message = "Hello, world!".to_string();
    let result = client_handle.send_message(42, sent_message.as_bytes());
    assert!(result.is_ok());

    let mut data: std::collections::btree_map::BTreeMap<String, sentry::protocol::Value> =
        std::collections::BTreeMap::new();
    data.insert("test_str_key".to_string(), "test_value".into());
    data.insert("test_int_key".to_string(), 1.into());

    let breadcrumb = sentry::protocol::Breadcrumb {
        timestamp: std::time::SystemTime::now(),
        ty: "debug".to_string(),
        level: sentry::protocol::Level::Info,
        category: Some("Test Category".to_string()),
        message: Some("Test Message".to_string()),
        data: data,
    };

    let serialized = serde_json::to_string(&breadcrumb).unwrap();
    let result = client_handle.send_message(43, serialized.as_bytes());
    assert!(result.is_ok());

    std::thread::sleep(std::time::Duration::from_secs(10));

    unsafe { sadness_generator::raise_segfault() };
}
