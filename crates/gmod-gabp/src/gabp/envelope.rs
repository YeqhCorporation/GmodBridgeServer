use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const GABP_VERSION: &str = "gabp/1";

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct GabpRequest {
    pub v: String,
    pub id: String,
    #[serde(rename = "type")]
    pub type_name: String,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GabpError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GabpResponse {
    pub v: String,
    pub id: String,
    #[serde(rename = "type")]
    pub type_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<GabpError>,
}

impl GabpResponse {
    pub fn success(id: String, result: Value) -> Self {
        Self {
            v: GABP_VERSION.to_string(),
            id,
            type_name: "response".to_string(),
            result: Some(result),
            error: None,
        }
    }

    pub fn failure(id: String, code: i32, message: impl Into<String>, data: Option<Value>) -> Self {
        Self {
            v: GABP_VERSION.to_string(),
            id,
            type_name: "response".to_string(),
            result: None,
            error: Some(GabpError {
                code,
                message: message.into(),
                data,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serializes_success_response_with_gabp_version() {
        let response = GabpResponse::success("abc".to_string(), json!({"ok": true}));
        let value = serde_json::to_value(response).unwrap();

        assert_eq!(value["v"], "gabp/1");
        assert_eq!(value["id"], "abc");
        assert_eq!(value["type"], "response");
        assert_eq!(value["result"]["ok"], true);
    }

    #[test]
    fn serializes_error_response_with_code_and_message() {
        let response = GabpResponse::failure("abc".to_string(), -32601, "Method not found", None);
        let value = serde_json::to_value(response).unwrap();

        assert_eq!(value["error"]["code"], -32601);
        assert_eq!(value["error"]["message"], "Method not found");
    }
}
