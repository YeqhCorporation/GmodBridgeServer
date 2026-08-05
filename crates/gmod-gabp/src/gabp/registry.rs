use crossbeam_channel::{unbounded, Receiver, Sender};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolDescriptor {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
    #[serde(rename = "outputSchema", skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingToolCall {
    pub request_id: String,
    pub tool_name: String,
    pub arguments: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToolCompletion {
    pub request_id: String,
    pub result: Result<Value, ToolFailure>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToolFailure {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RegistryError {
    InvalidToolName(String),
    ToolNotFound(String),
    DuplicateTool(String),
    LockPoisoned,
}

#[derive(Default)]
pub struct ToolRegistry {
    tools: Mutex<HashMap<String, ToolDescriptor>>,
}

impl ToolRegistry {
    pub fn register_tool(&self, descriptor: ToolDescriptor) -> Result<(), RegistryError> {
        if !is_valid_tool_name(&descriptor.name) {
            return Err(RegistryError::InvalidToolName(descriptor.name));
        }

        let mut tools = self.tools.lock().map_err(|_| RegistryError::LockPoisoned)?;
        if tools.contains_key(&descriptor.name) {
            return Err(RegistryError::DuplicateTool(descriptor.name));
        }

        tools.insert(descriptor.name.clone(), descriptor);
        Ok(())
    }

    pub fn has_tool(&self, name: &str) -> bool {
        self.tools
            .lock()
            .map(|tools| tools.contains_key(name))
            .unwrap_or(false)
    }

    pub fn list_tools(&self) -> Vec<ToolDescriptor> {
        let mut tools: Vec<_> = self
            .tools
            .lock()
            .map(|tools| tools.values().cloned().collect())
            .unwrap_or_default();
        tools.sort_by(|left, right| left.name.cmp(&right.name));
        tools
    }
}

#[derive(Clone)]
pub struct CallQueue {
    sender: Sender<PendingToolCall>,
    receiver: Receiver<PendingToolCall>,
    completions: Arc<Mutex<HashMap<String, ToolCompletion>>>,
}

impl Default for CallQueue {
    fn default() -> Self {
        let (sender, receiver) = unbounded();
        Self {
            sender,
            receiver,
            completions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl CallQueue {
    pub fn enqueue(&self, call: PendingToolCall) {
        let _ = self.sender.send(call);
    }

    pub fn try_dequeue(&self) -> Option<PendingToolCall> {
        self.receiver.try_recv().ok()
    }

    pub fn complete(&self, completion: ToolCompletion) -> Result<(), RegistryError> {
        let mut completions = self
            .completions
            .lock()
            .map_err(|_| RegistryError::LockPoisoned)?;
        completions.insert(completion.request_id.clone(), completion);
        Ok(())
    }

    pub fn wait_for_completion(
        &self,
        request_id: &str,
        timeout: Duration,
    ) -> Option<ToolCompletion> {
        let deadline = Instant::now() + timeout;

        loop {
            if let Ok(mut completions) = self.completions.lock() {
                if let Some(completion) = completions.remove(request_id) {
                    return Some(completion);
                }
            }

            if Instant::now() >= deadline {
                return None;
            }

            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

fn is_valid_tool_name(name: &str) -> bool {
    let segments: Vec<_> = name.split('/').collect();
    if segments.len() < 2 {
        return false;
    }

    segments.iter().all(|segment| {
        let mut chars = segment.chars();
        let Some(first) = chars.next() else {
            return false;
        };

        first.is_ascii_lowercase()
            && chars.all(|ch| {
                ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '-'
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn registers_and_lists_tool_descriptor() {
        let registry = ToolRegistry::default();
        registry
            .register_tool(ToolDescriptor {
                name: "server/status".to_string(),
                description: "Read server status.".to_string(),
                tags: vec!["read-only".to_string()],
                input_schema: json!({"type": "object", "properties": {}}),
                output_schema: Some(json!({"type": "object"})),
            })
            .unwrap();

        let tools = registry.list_tools();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "server/status");
    }

    #[test]
    fn rejects_invalid_tool_name() {
        let registry = ToolRegistry::default();
        let error = registry
            .register_tool(ToolDescriptor {
                name: "ServerStatus".to_string(),
                description: "Invalid name.".to_string(),
                tags: vec![],
                input_schema: json!({"type": "object"}),
                output_schema: None,
            })
            .unwrap_err();

        assert_eq!(error, RegistryError::InvalidToolName("ServerStatus".to_string()));
    }

    #[test]
    fn queues_and_completes_tool_call() {
        let queue = CallQueue::default();
        queue.enqueue(PendingToolCall {
            request_id: "req-1".to_string(),
            tool_name: "server/status".to_string(),
            arguments: json!({}),
        });

        let pending = queue.try_dequeue().unwrap();
        assert_eq!(pending.tool_name, "server/status");

        queue
            .complete(ToolCompletion {
                request_id: "req-1".to_string(),
                result: Ok(json!({"running": true})),
            })
            .unwrap();

        let completion = queue
            .wait_for_completion("req-1", Duration::from_millis(50))
            .unwrap();
        assert_eq!(completion.result.unwrap()["running"], true);
    }
}
