use gmod_gabp::gabp::frame::{encode_frame, FrameDecoder, MAX_FRAME_BYTES};
use gmod_gabp::gabp::runtime::{BridgeRuntime, RuntimeConfig};
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;

#[test]
fn fake_client_can_handshake_over_tcp() {
    let runtime = Arc::new(BridgeRuntime::new(RuntimeConfig {
        game_id: "gmod-dev".to_string(),
        port: 0,
        token: "secret".to_string(),
        launch_id: "tcp-test".to_string(),
    }));

    runtime.clone().start_listener().unwrap();
    let addr = runtime.local_addr().unwrap();

    let mut stream = TcpStream::connect(addr).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();

    let request = json!({
        "v": "gabp/1",
        "id": "hello-1",
        "type": "request",
        "method": "session/hello",
        "params": {
            "token": "secret",
            "bridgeVersion": "1.0.0",
            "platform": "windows",
            "launchId": "fake-client"
        }
    });

    let bytes = serde_json::to_vec(&request).unwrap();
    stream.write_all(&encode_frame(&bytes)).unwrap();

    let mut decoder = FrameDecoder::new(MAX_FRAME_BYTES);
    let mut buffer = [0_u8; 4096];
    let read = stream.read(&mut buffer).unwrap();
    let messages = decoder.push(&buffer[..read]).unwrap();
    let response: Value = serde_json::from_slice(&messages[0]).unwrap();

    assert_eq!(response["id"], "hello-1");
    assert_eq!(response["result"]["agentId"], "gmod-dev");
}
