use crate::gabp::envelope::{GabpRequest, GabpResponse, GABP_VERSION};
use crate::gabp::frame::{encode_frame, FrameDecoder, MAX_FRAME_BYTES};
use crate::gabp::registry::{
    CallQueue, PendingToolCall, ToolCompletion, ToolFailure, ToolRegistry,
};
use gmod::lua::State;
use once_cell::sync::Lazy;
use serde_json::{json, Value};
use std::env;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeConfig {
    pub game_id: String,
    pub port: u16,
    pub token: String,
    pub launch_id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeError {
    MissingEnv(&'static str),
    InvalidPort(String),
    ListenFailed(String),
}

pub struct BridgeRuntime {
    config: RuntimeConfig,
    registry: ToolRegistry,
    queue: CallQueue,
    tool_timeout: Duration,
    listener_addr: Mutex<Option<SocketAddr>>,
}

static GLOBAL_RUNTIME: Lazy<Mutex<Option<Arc<BridgeRuntime>>>> = Lazy::new(|| Mutex::new(None));

impl RuntimeConfig {
    pub fn from_env() -> Result<Self, RuntimeError> {
        let game_id =
            env::var("GABS_GAME_ID").map_err(|_| RuntimeError::MissingEnv("GABS_GAME_ID"))?;
        let port_raw = env::var("GABP_SERVER_PORT")
            .map_err(|_| RuntimeError::MissingEnv("GABP_SERVER_PORT"))?;
        let token = env::var("GABP_TOKEN").map_err(|_| RuntimeError::MissingEnv("GABP_TOKEN"))?;
        let port = port_raw
            .parse::<u16>()
            .map_err(|_| RuntimeError::InvalidPort(port_raw.clone()))?;
        let launch_id =
            env::var("GABP_LAUNCH_ID").unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());

        Ok(Self {
            game_id,
            port,
            token,
            launch_id,
        })
    }
}

impl BridgeRuntime {
    pub fn new(config: RuntimeConfig) -> Self {
        Self {
            config,
            registry: ToolRegistry::default(),
            queue: CallQueue::default(),
            tool_timeout: Duration::from_secs(30),
            listener_addr: Mutex::new(None),
        }
    }

    pub fn registry(&self) -> &ToolRegistry {
        &self.registry
    }

    pub fn queue(&self) -> &CallQueue {
        &self.queue
    }

    pub fn handle_request(&self, request: GabpRequest) -> GabpResponse {
        if request.v != GABP_VERSION || request.type_name != "request" {
            return GabpResponse::failure(request.id, -32600, "Invalid GABP request", None);
        }

        match request.method.as_str() {
            "session/hello" => self.handle_session_hello(request.id, request.params),
            "tools/list" => {
                GabpResponse::success(request.id, json!({ "tools": self.registry.list_tools() }))
            }
            "tools/call" => self.handle_tools_call(request.id, request.params),
            method => GabpResponse::failure(
                request.id,
                -32601,
                "Method not found",
                Some(json!({ "method": method })),
            ),
        }
    }

    pub fn start_listener(self: Arc<Self>) -> Result<(), RuntimeError> {
        let listener = TcpListener::bind(("127.0.0.1", self.config.port))
            .map_err(|err| RuntimeError::ListenFailed(err.to_string()))?;
        let addr = listener
            .local_addr()
            .map_err(|err| RuntimeError::ListenFailed(err.to_string()))?;

        if let Ok(mut listener_addr) = self.listener_addr.lock() {
            *listener_addr = Some(addr);
        }

        thread::spawn(move || {
            for incoming in listener.incoming() {
                match incoming {
                    Ok(stream) => {
                        let runtime = self.clone();
                        thread::spawn(move || runtime.handle_connection(stream));
                    }
                    Err(err) => {
                        eprintln!("[gmod-gabp] accept failed: {err}");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    pub fn local_addr(&self) -> Option<SocketAddr> {
        self.listener_addr.lock().ok().and_then(|addr| *addr)
    }

    fn handle_session_hello(&self, id: String, params: Value) -> GabpResponse {
        let token = params.get("token").and_then(Value::as_str).unwrap_or("");
        if token != self.config.token {
            return GabpResponse::failure(id, -32101, "Authentication failed", None);
        }

        GabpResponse::success(
            id,
            json!({
                "agentId": self.config.game_id,
                "app": {
                    "name": "Garry's Mod GABP Bridge",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "capabilities": {
                    "methods": ["tools/list", "tools/call"],
                    "events": [],
                    "resources": []
                },
                "schemaVersion": "1.0"
            }),
        )
    }

    fn handle_tools_call(&self, id: String, params: Value) -> GabpResponse {
        let Some(name) = params.get("name").and_then(Value::as_str) else {
            return GabpResponse::failure(id, -32602, "Missing tool name", None);
        };

        if !self.registry.has_tool(name) {
            return GabpResponse::failure(
                id,
                -32400,
                "Tool not found",
                Some(json!({ "tool": name })),
            );
        }

        let arguments = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
        self.queue.enqueue(PendingToolCall {
            request_id: id.clone(),
            tool_name: name.to_string(),
            arguments,
        });

        match self.queue.wait_for_completion(&id, self.tool_timeout) {
            Some(completion) => match completion.result {
                Ok(value) => GabpResponse::success(id, value),
                Err(ToolFailure {
                    code,
                    message,
                    data,
                }) => GabpResponse::failure(id, code, message, data),
            },
            None => GabpResponse::failure(id, -32403, "Tool execution timed out", None),
        }
    }

    fn handle_connection(&self, mut stream: TcpStream) {
        let mut decoder = FrameDecoder::new(MAX_FRAME_BYTES);
        let mut buffer = [0_u8; 8192];

        loop {
            let read = match stream.read(&mut buffer) {
                Ok(0) => return,
                Ok(read) => read,
                Err(err) => {
                    eprintln!("[gmod-gabp] read failed: {err}");
                    return;
                }
            };

            let messages = match decoder.push(&buffer[..read]) {
                Ok(messages) => messages,
                Err(err) => {
                    eprintln!("[gmod-gabp] frame error: {err:?}");
                    return;
                }
            };

            for message in messages {
                let response = match serde_json::from_slice::<GabpRequest>(&message) {
                    Ok(request) => self.handle_request(request),
                    Err(err) => GabpResponse::failure(
                        "00000000-0000-0000-0000-000000000000".to_string(),
                        -32700,
                        "Parse error",
                        Some(json!({ "error": err.to_string() })),
                    ),
                };

                let Ok(bytes) = serde_json::to_vec(&response) else {
                    return;
                };

                if stream.write_all(&encode_frame(&bytes)).is_err() {
                    return;
                }
            }
        }
    }
}

pub unsafe fn install_lua_api(lua: State) {
    lua.new_table();

    lua.push_function(lua_start);
    lua.set_field(-2, gmod::lua_string!("start"));

    lua.push_function(lua_stop);
    lua.set_field(-2, gmod::lua_string!("stop"));

    lua.push_function(lua_register_tool_native);
    lua.set_field(-2, gmod::lua_string!("register_tool_native"));

    lua.push_function(lua_poll_call_native);
    lua.set_field(-2, gmod::lua_string!("poll_call_native"));

    lua.push_function(lua_complete_call_native);
    lua.set_field(-2, gmod::lua_string!("complete_call_native"));

    lua.push_function(lua_fail_call_native);
    lua.set_field(-2, gmod::lua_string!("fail_call_native"));

    lua.set_global(gmod::lua_string!("gabp"));
}

#[gmod::lua_function]
unsafe fn lua_start(lua: State) -> i32 {
    match start_global_runtime_from_env() {
        Ok(()) => {
            lua.push_boolean(true);
            1
        }
        Err(err) => {
            lua.push_boolean(false);
            lua.push_string(&err);
            2
        }
    }
}

#[gmod::lua_function]
unsafe fn lua_stop(lua: State) -> i32 {
    shutdown_global_runtime();
    lua.push_boolean(true);
    1
}

#[gmod::lua_function]
unsafe fn lua_register_tool_native(lua: State) -> i32 {
    let name = lua.check_string(1).to_string();
    let descriptor_json = lua.check_string(2).to_string();
    let result = register_tool_from_lua(&name, &descriptor_json);
    push_lua_result(lua, result);
    2
}

#[gmod::lua_function]
unsafe fn lua_poll_call_native(lua: State) -> i32 {
    match poll_call_for_lua() {
        Some(call_json) => lua.push_string(&call_json),
        None => lua.push_nil(),
    }
    1
}

#[gmod::lua_function]
unsafe fn lua_complete_call_native(lua: State) -> i32 {
    let request_id = lua.check_string(1).to_string();
    let result_json = lua.check_string(2).to_string();
    let result = complete_call_from_lua(&request_id, &result_json);
    push_lua_result(lua, result);
    2
}

#[gmod::lua_function]
unsafe fn lua_fail_call_native(lua: State) -> i32 {
    let request_id = lua.check_string(1).to_string();
    let code = lua.check_number(2) as i32;
    let message = lua.check_string(3).to_string();
    let data_json = if lua.is_none_or_nil(4) {
        None
    } else {
        Some(lua.check_string(4).to_string())
    };
    let result = fail_call_from_lua(&request_id, code, &message, data_json.as_deref());
    push_lua_result(lua, result);
    2
}

unsafe fn push_lua_result(lua: State, result: Result<(), String>) {
    match result {
        Ok(()) => {
            lua.push_boolean(true);
            lua.push_nil();
        }
        Err(err) => {
            lua.push_boolean(false);
            lua.push_string(&err);
        }
    }
}

pub fn start_global_runtime_from_env() -> Result<(), String> {
    let config = RuntimeConfig::from_env().map_err(|err| format!("{err:?}"))?;
    let runtime = Arc::new(BridgeRuntime::new(config));
    runtime
        .clone()
        .start_listener()
        .map_err(|err| format!("{err:?}"))?;
    set_global_runtime(runtime)
}

pub fn set_global_runtime(runtime: Arc<BridgeRuntime>) -> Result<(), String> {
    let mut global = GLOBAL_RUNTIME
        .lock()
        .map_err(|_| "runtime lock poisoned".to_string())?;
    if global.is_some() {
        return Err("runtime already started".to_string());
    }
    *global = Some(runtime);
    Ok(())
}

pub fn register_tool_from_lua(name: &str, descriptor_json: &str) -> Result<(), String> {
    let descriptor = crate::gabp::lua_bridge::parse_tool_descriptor(name, descriptor_json)
        .map_err(|err| format!("{err:?}"))?;
    let runtime = current_runtime()?;
    runtime
        .registry()
        .register_tool(descriptor)
        .map_err(|err| format!("{err:?}"))
}

pub fn poll_call_for_lua() -> Option<String> {
    let runtime = current_runtime().ok()?;
    let call = runtime.queue().try_dequeue()?;
    crate::gabp::lua_bridge::pending_call_to_json(call).ok()
}

pub fn complete_call_from_lua(request_id: &str, result_json: &str) -> Result<(), String> {
    let result = serde_json::from_str(result_json).map_err(|err| err.to_string())?;
    let runtime = current_runtime()?;
    runtime
        .queue()
        .complete(ToolCompletion {
            request_id: request_id.to_string(),
            result: Ok(result),
        })
        .map_err(|err| format!("{err:?}"))
}

pub fn fail_call_from_lua(
    request_id: &str,
    code: i32,
    message: &str,
    data_json: Option<&str>,
) -> Result<(), String> {
    let data = match data_json {
        Some(raw) => Some(serde_json::from_str(raw).map_err(|err| err.to_string())?),
        None => None,
    };
    let runtime = current_runtime()?;
    runtime
        .queue()
        .complete(ToolCompletion {
            request_id: request_id.to_string(),
            result: Err(ToolFailure {
                code,
                message: message.to_string(),
                data,
            }),
        })
        .map_err(|err| format!("{err:?}"))
}

fn current_runtime() -> Result<Arc<BridgeRuntime>, String> {
    GLOBAL_RUNTIME
        .lock()
        .map_err(|_| "runtime lock poisoned".to_string())?
        .clone()
        .ok_or_else(|| "runtime not started".to_string())
}

pub fn shutdown_global_runtime() {
    if let Ok(mut global) = GLOBAL_RUNTIME.lock() {
        *global = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gabp::envelope::GabpRequest;
    use serde_json::json;

    static GLOBAL_RUNTIME_TEST_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    fn runtime() -> BridgeRuntime {
        BridgeRuntime::new(RuntimeConfig {
            game_id: "gmod-dev".to_string(),
            port: 39001,
            token: "secret".to_string(),
            launch_id: "test-launch".to_string(),
        })
    }

    #[test]
    fn accepts_valid_session_hello() {
        let response = runtime().handle_request(GabpRequest {
            v: "gabp/1".to_string(),
            id: "hello-1".to_string(),
            type_name: "request".to_string(),
            method: "session/hello".to_string(),
            params: json!({
                "token": "secret",
                "bridgeVersion": "1.0.0",
                "platform": "windows",
                "launchId": "client-launch"
            }),
        });

        let value = serde_json::to_value(response).unwrap();
        assert_eq!(value["result"]["agentId"], "gmod-dev");
        assert_eq!(value["result"]["app"]["name"], "Garry's Mod GABP Bridge");
        assert_eq!(value["result"]["capabilities"]["methods"][0], "tools/list");
    }

    #[test]
    fn rejects_bad_token() {
        let response = runtime().handle_request(GabpRequest {
            v: "gabp/1".to_string(),
            id: "hello-1".to_string(),
            type_name: "request".to_string(),
            method: "session/hello".to_string(),
            params: json!({"token": "wrong"}),
        });

        let value = serde_json::to_value(response).unwrap();
        assert_eq!(value["error"]["code"], -32101);
    }

    #[test]
    fn registers_tool_from_lua_json_against_global_runtime() {
        let _guard = GLOBAL_RUNTIME_TEST_LOCK.lock().unwrap();
        shutdown_global_runtime();
        set_global_runtime(Arc::new(runtime())).unwrap();

        register_tool_from_lua(
            "server/status",
            r#"{"description":"Read status","tags":["read-only"],"inputSchema":{"type":"object"}}"#,
        )
        .unwrap();

        let tools = current_runtime().unwrap().registry().list_tools();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "server/status");

        shutdown_global_runtime();
    }

    #[test]
    fn polls_and_completes_call_for_lua() {
        let _guard = GLOBAL_RUNTIME_TEST_LOCK.lock().unwrap();
        shutdown_global_runtime();
        let runtime = Arc::new(runtime());
        runtime.queue().enqueue(PendingToolCall {
            request_id: "req-1".to_string(),
            tool_name: "server/status".to_string(),
            arguments: json!({"verbose": true}),
        });
        set_global_runtime(runtime.clone()).unwrap();

        let call_json = poll_call_for_lua().unwrap();
        assert!(call_json.contains(r#""requestId":"req-1""#));

        complete_call_from_lua("req-1", r#"{"ok":true}"#).unwrap();
        let completion = runtime
            .queue()
            .wait_for_completion("req-1", Duration::from_millis(50))
            .unwrap();
        assert_eq!(completion.result.unwrap()["ok"], true);

        shutdown_global_runtime();
    }
}
