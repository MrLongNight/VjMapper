use crate::protocol::*;
use crate::McpAction;
use anyhow::Result;
use crossbeam_channel::Sender;
use mapmap_control::osc::client::OscClient;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{error, info};

pub struct McpServer {
    // Optional OSC client (currently unused but will be used for OSC tools)
    #[allow(dead_code)]
    osc_client: Option<OscClient>,
    // Channel to send actions to main app
    action_sender: Option<Sender<McpAction>>,
}

impl McpServer {
    pub fn new(action_sender: Option<crossbeam_channel::Sender<crate::McpAction>>) -> Self {
        // Try to connect to default VJMapper OSC port
        let osc_client = match OscClient::new("127.0.0.1:8000") {
            Ok(client) => {
                info!("MCP Server connected to OSC at 127.0.0.1:8000");
                Some(client)
            }
            Err(e) => {
                error!("Failed to create OSC client: {}", e);
                None
            }
        };
        Self {
            osc_client,
            action_sender,
        }
    }

    pub async fn run_stdio(&self) -> Result<()> {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line).await?;
            if bytes_read == 0 {
                break; // EOF
            }

            let response = self.handle_request(&line).await;

            if let Some(resp) = response {
                let json = serde_json::to_string(&resp)?;
                stdout.write_all(json.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
            }
        }

        Ok(())
    }

    async fn handle_request(&self, request_str: &str) -> Option<JsonRpcResponse> {
        let request: JsonRpcRequest = match serde_json::from_str(request_str) {
            Ok(req) => req,
            Err(e) => {
                error!("Failed to parse JSON-RPC request: {}", e);
                return Some(error_response(None, -32700, "Parse error"));
            }
        };

        let id = request.id.clone();

        match request.method.as_str() {
            "initialize" => {
                let result = InitializeResult {
                    protocol_version: "2024-11-05".to_string(),
                    capabilities: ServerCapabilities {
                        tools: Some(serde_json::json!({
                            "listChanged": true
                        })),
                        resources: None,
                        prompts: None,
                    },
                    server_info: ServerInfo {
                        name: "vjmapper-mcp".to_string(),
                        version: "0.1.0".to_string(),
                    },
                };
                Some(success_response(id, serde_json::to_value(result).unwrap()))
            }
            "notifications/initialized" => None,
            "tools/list" => {
                let tools = vec![
                    Tool {
                        name: "send_osc".to_string(),
                        description: Some(
                            "Send an Open Sound Control (OSC) message to the running VJMapper instance"
                                .to_string(),
                        ),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "address": {
                                    "type": "string",
                                    "description": "OSC Address (e.g. /mapmap/layer/1/opacity)"
                                },
                                "args": {
                                    "type": "array",
                                    "items": { "type": "number" },
                                    "description": "List of numeric arguments (floats)"
                                }
                            },
                            "required": ["address", "args"]
                        }),
                    },
                    Tool {
                        name: "layer_set_opacity".to_string(),
                        description: Some("Set the opacity of a layer".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "layer_id": { "type": "integer" },
                                "opacity": { "type": "number", "minimum": 0.0, "maximum": 1.0 }
                            },
                            "required": ["layer_id", "opacity"]
                        }),
                    },
                    Tool {
                        name: "layer_set_visibility".to_string(),
                        description: Some("Set the visibility of a layer".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "layer_id": { "type": "integer" },
                                "visible": { "type": "boolean" }
                            },
                            "required": ["layer_id", "visible"]
                        }),
                    },
                    Tool {
                        name: "layer_create".to_string(),
                        description: Some("Create a new layer".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "name": { "type": "string", "description": "Optional name for the new layer" }
                            },
                        }),
                    },
                    Tool {
                        name: "layer_delete".to_string(),
                        description: Some("Delete a layer".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "layer_id": { "type": "integer" }
                            },
                            "required": ["layer_id"]
                        }),
                    },
                    Tool {
                        name: "cue_trigger".to_string(),
                        description: Some("Trigger a specific cue".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "cue_id": { "type": "integer" }
                            },
                            "required": ["cue_id"]
                        }),
                    },
                    Tool {
                        name: "cue_next".to_string(),
                        description: Some("Go to the next cue".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {},
                        }),
                    },
                    Tool {
                        name: "cue_previous".to_string(),
                        description: Some("Go to the previous cue".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {},
                        }),
                    },
                    Tool {
                        name: "media_play".to_string(),
                        description: Some("Start media playback".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {},
                        }),
                    },
                    Tool {
                        name: "media_pause".to_string(),
                        description: Some("Pause media playback".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {},
                        }),
                    },
                    Tool {
                        name: "media_stop".to_string(),
                        description: Some("Stop media playback".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {},
                        }),
                    },
                    Tool {
                        name: "project_save".to_string(),
                        description: Some("Save the current project".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "path": { "type": "string" }
                            },
                            "required": ["path"]
                        }),
                    },
                    Tool {
                        name: "project_load".to_string(),
                        description: Some("Load a project from disk".to_string()),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "path": { "type": "string" }
                            },
                            "required": ["path"]
                        }),
                    },
                ];

                Some(success_response(
                    id,
                    serde_json::json!({
                        "tools": tools
                    }),
                ))
            }
            "resources/list" => {
                let resources = vec![
                    serde_json::json!({
                        "uri": "project://current",
                        "name": "Current Project",
                        "mimeType": "application/json",
                        "description": "The current VJMapper project state"
                    }),
                    serde_json::json!({
                        "uri": "layer://list",
                        "name": "Layer List",
                        "mimeType": "application/json",
                        "description": "List of all layers"
                    }),
                ];
                Some(success_response(
                    id,
                    serde_json::json!({ "resources": resources }),
                ))
            }
            "resources/read" => {
                // Parse params
                let params: Option<serde_json::Value> =
                    serde_json::from_value(request.params.unwrap_or(serde_json::Value::Null)).ok();
                let uri = params
                    .and_then(|p| p.get("uri").and_then(|v| v.as_str()).map(|s| s.to_string()));

                if let Some(uri) = uri {
                    match uri.as_str() {
                        "project://current" => {
                            // TODO: Implement shared state reading
                            Some(success_response(
                                id,
                                serde_json::json!({
                                    "contents": [{
                                        "uri": uri,
                                        "mimeType": "application/json",
                                        "text": "{\"error\": \"Shared state access not yet implemented\"}"
                                    }]
                                }),
                            ))
                        }
                        _ => Some(error_response(id, -32602, "Resource not found")),
                    }
                } else {
                    Some(error_response(id, -32602, "Missing uri parameter"))
                }
            }
            "prompts/list" => {
                let prompts = vec![
                    serde_json::json!({
                        "name": "create_mapping",
                        "description": "Assist in creating a new projection mapping",
                        "arguments": []
                    }),
                    serde_json::json!({
                         "name": "troubleshoot",
                         "description": "Diagnose common problems",
                         "arguments": []
                    }),
                ];
                Some(success_response(
                    id,
                    serde_json::json!({ "prompts": prompts }),
                ))
            }
            "prompts/get" => {
                let params: Option<serde_json::Value> =
                    serde_json::from_value(request.params.unwrap_or(serde_json::Value::Null)).ok();
                let name = params.and_then(|p| {
                    p.get("name")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                });

                if let Some(name_str) = name {
                    match name_str.as_str() {
                        "create_mapping" => Some(success_response(
                            id,
                            serde_json::json!({
                                "description": "Create a new mapping",
                                "messages": [
                                    {
                                        "role": "user",
                                        "content": {
                                            "type": "text",
                                            "text": "I want to create a new mapping for a surface. Please guide me through the steps affecting layers and meshes."
                                        }
                                    }
                                ]
                            }),
                        )),
                        "troubleshoot" => Some(success_response(
                            id,
                            serde_json::json!({
                                "description": "Troubleshoot VJMapper",
                                "messages": [
                                    {
                                        "role": "user",
                                        "content": {
                                            "type": "text",
                                            "text": "Analyze the current state and logs for any errors or misconfigurations."
                                        }
                                    }
                                ]
                            }),
                        )),
                        _ => Some(error_response(id, -32601, "Prompt not found")),
                    }
                } else {
                    Some(error_response(id, -32602, "Missing name parameter"))
                }
            }
            // Handle tool calls
            "tools/call" => {
                // Parse params
                let params: CallToolParams = match serde_json::from_value(
                    request.params.clone().unwrap_or(serde_json::Value::Null),
                ) {
                    Ok(p) => p,
                    Err(_) => return Some(error_response(id, -32602, "Invalid params")),
                };

                match params.name.as_str() {
                    "project_save" => {
                        if let Some(args) = params.arguments {
                            if let Some(path_val) = args.get("path") {
                                if let Some(path_str) = path_val.as_str() {
                                    if let Some(sender) = &self.action_sender {
                                        let _ = sender.send(crate::McpAction::SaveProject(
                                            PathBuf::from(path_str),
                                        ));
                                    }
                                    return Some(success_response(
                                        id,
                                        serde_json::json!({"status":"queued"}),
                                    ));
                                }
                            }
                        }
                        Some(error_response(id, -32602, "Missing path"))
                    }
                    "project_load" => {
                        if let Some(args) = params.arguments {
                            if let Some(path_val) = args.get("path") {
                                if let Some(path_str) = path_val.as_str() {
                                    if let Some(sender) = &self.action_sender {
                                        let _ = sender.send(crate::McpAction::LoadProject(
                                            PathBuf::from(path_str),
                                        ));
                                    }
                                    return Some(success_response(
                                        id,
                                        serde_json::json!({"status":"queued"}),
                                    ));
                                }
                            }
                        }
                        Some(error_response(id, -32602, "Missing path"))
                    }
                    "layer_create" => {
                        if let Some(args) = params.arguments {
                            if let Some(name_val) = args.get("name") {
                                if let Some(name_str) = name_val.as_str() {
                                    if let Some(sender) = &self.action_sender {
                                        let _ = sender
                                            .send(crate::McpAction::AddLayer(name_str.to_string()));
                                    }
                                    return Some(success_response(
                                        id,
                                        serde_json::json!({"status":"queued"}),
                                    ));
                                }
                            }
                        }
                        Some(error_response(id, -32602, "Missing layer name"))
                    }
                    "layer_delete" => {
                        if let Some(args) = params.arguments {
                            if let Some(layer_id_val) = args.get("layer_id") {
                                if let Some(layer_id) = layer_id_val.as_u64() {
                                    if let Some(sender) = &self.action_sender {
                                        let _ = sender
                                            .send(crate::McpAction::RemoveLayer(layer_id as u32));
                                    }
                                    return Some(success_response(
                                        id,
                                        serde_json::json!({"status":"queued"}),
                                    ));
                                }
                            }
                        }
                        Some(error_response(id, -32602, "Missing layer_id"))
                    }
                    "cue_trigger" => {
                        if let Some(args) = params.arguments {
                            if let Some(cue_id_val) = args.get("cue_id") {
                                if let Some(cue_id) = cue_id_val.as_u64() {
                                    if let Some(sender) = &self.action_sender {
                                        let _ = sender
                                            .send(crate::McpAction::TriggerCue(cue_id as u32));
                                    }
                                    return Some(success_response(
                                        id,
                                        serde_json::json!({"status":"queued"}),
                                    ));
                                }
                            }
                        }
                        Some(error_response(id, -32602, "Missing cue_id"))
                    }
                    "cue_next" => {
                        if let Some(sender) = &self.action_sender {
                            let _ = sender.send(crate::McpAction::NextCue);
                        }
                        Some(success_response(id, serde_json::json!({"status":"queued"})))
                    }
                    "cue_previous" => {
                        if let Some(sender) = &self.action_sender {
                            let _ = sender.send(crate::McpAction::PrevCue);
                        }
                        Some(success_response(id, serde_json::json!({"status":"queued"})))
                    }
                    "media_play" => {
                        if let Some(sender) = &self.action_sender {
                            let _ = sender.send(crate::McpAction::MediaPlay);
                        }
                        Some(success_response(id, serde_json::json!({"status":"queued"})))
                    }
                    "media_pause" => {
                        if let Some(sender) = &self.action_sender {
                            let _ = sender.send(crate::McpAction::MediaPause);
                        }
                        Some(success_response(id, serde_json::json!({"status":"queued"})))
                    }
                    "media_stop" => {
                        if let Some(sender) = &self.action_sender {
                            let _ = sender.send(crate::McpAction::MediaStop);
                        }
                        Some(success_response(id, serde_json::json!({"status":"queued"})))
                    }
                    "layer_list" => {
                        // Mock empty list for now
                        let layers: Vec<String> = vec![];
                        Some(success_response(id, serde_json::json!({"layers": layers})))
                    }
                    _ => Some(error_response(id, -32601, "Tool not found")),
                }
            }
            _ => Some(error_response(id, -32601, "Method not found")),
        }
    }

    #[allow(dead_code)]
    fn handle_send_osc(
        &self,
        id: Option<serde_json::Value>,
        args: &serde_json::Value,
    ) -> Option<JsonRpcResponse> {
        if let (Some(address_val), Some(args_val)) = (args.get("address"), args.get("args")) {
            if let (Some(address), Some(args_array)) = (address_val.as_str(), args_val.as_array()) {
                let mut osc_args = Vec::new();
                for arg in args_array {
                    if let Some(f) = arg.as_f64() {
                        osc_args.push(rosc::OscType::Float(f as f32));
                    }
                }
                return self.send_osc_msg(address, osc_args, id);
            }
        }
        Some(error_response(
            id,
            -32602,
            "Missing address or args argument",
        ))
    }

    #[allow(dead_code)]
    fn send_osc_msg(
        &self,
        address: &str,
        args: Vec<rosc::OscType>,
        id: Option<serde_json::Value>,
    ) -> Option<JsonRpcResponse> {
        if let Some(client) = &self.osc_client {
            match client.send_message(address, args) {
                Ok(_) => Some(success_response(
                    id,
                    serde_json::json!(CallToolResult {
                        content: vec![ToolContent::Text {
                            text: format!("Sent OSC message to {}", address)
                        }],
                        is_error: Some(false)
                    }),
                )),
                Err(e) => Some(success_response(
                    id,
                    serde_json::json!(CallToolResult {
                        content: vec![ToolContent::Text {
                            text: format!("OSC Error: {}", e)
                        }],
                        is_error: Some(true)
                    }),
                )),
            }
        } else {
            Some(error_response(id, -32000, "OSC Client not initialized"))
        }
    }
}

fn success_response(id: Option<serde_json::Value>, result: serde_json::Value) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(result),
        error: None,
        id,
    }
}

fn error_response(id: Option<serde_json::Value>, code: i32, message: &str) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: None,
        error: Some(JsonRpcError {
            code,
            message: message.to_string(),
            data: None,
        }),
        id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam_channel::unbounded;
    use serde_json::json;

    #[tokio::test]
    async fn test_handle_layer_create() {
        let (tx, rx) = unbounded();
        let server = McpServer::new(Some(tx));

        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "layer_create",
                "arguments": {
                    "name": "Test Layer"
                }
            }
        });

        let response = server.handle_request(&request.to_string()).await;
        assert!(response.is_some());

        let action = rx.try_recv().unwrap();
        if let McpAction::AddLayer(name) = action {
            assert_eq!(name, "Test Layer");
        } else {
            panic!("Expected AddLayer action");
        }
    }

    #[tokio::test]
    async fn test_handle_layer_delete() {
        let (tx, rx) = unbounded();
        let server = McpServer::new(Some(tx));

        let request = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "layer_delete",
                "arguments": {
                    "layer_id": 42
                }
            }
        });

        server.handle_request(&request.to_string()).await;
        let action = rx.try_recv().unwrap();
        if let McpAction::RemoveLayer(id) = action {
            assert_eq!(id, 42);
        } else {
            panic!("Expected RemoveLayer action");
        }
    }

    #[tokio::test]
    async fn test_handle_cue_trigger() {
        let (tx, rx) = unbounded();
        let server = McpServer::new(Some(tx));

        let request = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "cue_trigger",
                "arguments": {
                    "cue_id": 5
                }
            }
        });

        server.handle_request(&request.to_string()).await;
        let action = rx.try_recv().unwrap();
        if let McpAction::TriggerCue(id) = action {
            assert_eq!(id, 5);
        } else {
            panic!("Expected TriggerCue action");
        }
    }

    #[tokio::test]
    async fn test_handle_cue_navigation() {
        let (tx, rx) = unbounded();
        let server = McpServer::new(Some(tx));

        // Test Next
        let next_req = json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "cue_next",
                "arguments": {}
            }
        });
        server.handle_request(&next_req.to_string()).await;
        assert!(matches!(rx.try_recv().unwrap(), McpAction::NextCue));

        // Test Previous
        let prev_req = json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "tools/call",
            "params": {
                "name": "cue_previous",
                "arguments": {}
            }
        });
        server.handle_request(&prev_req.to_string()).await;
        assert!(matches!(rx.try_recv().unwrap(), McpAction::PrevCue));
    }

    #[tokio::test]
    async fn test_handle_project_save_load() {
        let (tx, rx) = unbounded();
        let server = McpServer::new(Some(tx));

        // Test Save
        let save_req = json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "tools/call",
            "params": {
                "name": "project_save",
                "arguments": {
                    "path": "test.mapmap"
                }
            }
        });
        server.handle_request(&save_req.to_string()).await;
        let action = rx.try_recv().unwrap();
        if let McpAction::SaveProject(path) = action {
            assert_eq!(path.to_str().unwrap(), "test.mapmap");
        } else {
            panic!("Expected SaveProject action");
        }

        // Test Load
        let load_req = json!({
            "jsonrpc": "2.0",
            "id": 7,
            "method": "tools/call",
            "params": {
                "name": "project_load",
                "arguments": {
                    "path": "other.mapmap"
                }
            }
        });
        server.handle_request(&load_req.to_string()).await;
        let action = rx.try_recv().unwrap();
        if let McpAction::LoadProject(path) = action {
            assert_eq!(path.to_str().unwrap(), "other.mapmap");
        } else {
            panic!("Expected LoadProject action");
        }
    }

    #[tokio::test]
    async fn test_handle_send_osc() {
        let (tx, _rx) = unbounded();
        let server = McpServer::new(Some(tx));

        let request = json!({
            "jsonrpc": "2.0",
            "id": 8,
            "method": "tools/call",
            "params": {
                "name": "send_osc",
                "arguments": {
                    "address": "/test/addr",
                    "args": ["hello", 123, 1.5]
                }
            }
        });

        let response = server.handle_request(&request.to_string()).await;
        assert!(response.is_some());
        let resp = response.unwrap();
        assert!(
            resp.error.is_none(),
            "Response should not be an error: {:?}",
            resp.error
        );

        let result = resp.result.unwrap();
        // result is a CallToolResult
        assert_eq!(result["isError"], false);
        assert!(result["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Sent OSC"));
    }
}
