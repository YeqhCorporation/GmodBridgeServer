use crate::gabp::registry::{PendingToolCall, ToolDescriptor};
use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq)]
pub enum LuaBridgeError {
    InvalidJson(String),
    MissingDescription,
}

pub fn parse_tool_descriptor(
    name: &str,
    descriptor_json: &str,
) -> Result<ToolDescriptor, LuaBridgeError> {
    let value: Value = serde_json::from_str(descriptor_json)
        .map_err(|err| LuaBridgeError::InvalidJson(err.to_string()))?;
    let description = value
        .get("description")
        .and_then(Value::as_str)
        .ok_or(LuaBridgeError::MissingDescription)?
        .to_string();
    let tags = value
        .get("tags")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let input_schema = value
        .get("inputSchema")
        .cloned()
        .unwrap_or_else(|| json!({ "type": "object", "properties": {} }));
    let output_schema = value.get("outputSchema").cloned();

    Ok(ToolDescriptor {
        name: name.to_string(),
        description,
        tags,
        input_schema,
        output_schema,
    })
}

pub fn pending_call_to_json(call: PendingToolCall) -> Result<String, serde_json::Error> {
    serde_json::to_string(&json!({
        "requestId": call.request_id,
        "toolName": call.tool_name,
        "arguments": call.arguments
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tool_descriptor_from_lua_json() {
        let descriptor = parse_tool_descriptor(
            "server/status",
            r#"{"description":"Read status","tags":["read-only"],"inputSchema":{"type":"object"}}"#,
        )
        .unwrap();

        assert_eq!(descriptor.name, "server/status");
        assert_eq!(descriptor.description, "Read status");
        assert_eq!(descriptor.tags, vec!["read-only"]);
    }

    #[test]
    fn serializes_pending_call_for_lua() {
        let json = pending_call_to_json(PendingToolCall {
            request_id: "req-1".to_string(),
            tool_name: "server/status".to_string(),
            arguments: json!({"verbose": true}),
        })
        .unwrap();

        assert!(json.contains(r#""requestId":"req-1""#));
        assert!(json.contains(r#""toolName":"server/status""#));
    }
}
