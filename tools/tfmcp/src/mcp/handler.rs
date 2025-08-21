use crate::core::tfmcp::{JsonRpcErrorCode, TfMcp, TfMcpError};
use crate::mcp::stdio::{Message, StdioTransport, Transport};
use crate::terraform::model::TerraformAnalysis;
use futures::StreamExt;
use serde_json::{json, Value};
use std::path::PathBuf;
use crate::shared::logging;

const TOOLS_JSON: &str = r#"{
  "tools": [
    {
      "name": "list_terraform_resources",
      "description": "List all resources defined in the Terraform project",
      "inputSchema": {
        "type": "object",
        "properties": {}
      },
      "outputSchema": {
        "type": "object",
        "properties": {
          "resources": {
            "type": "array",
            "items": {
              "type": "string"
            },
            "description": "List of resource identifiers"
          }
        },
        "required": ["resources"]
      }
    },
    {
      "name": "analyze_terraform",
      "description": "Analyze Terraform configuration files and provide detailed information",
      "inputSchema": {
        "type": "object",
        "properties": {
          "path": {
            "type": "string",
            "description": "Path to the Terraform configuration directory (optional)"
          }
        }
      },
      "outputSchema": {
        "type": "object",
        "properties": {
          "analysis": {
            "type": "object",
            "properties": {
              "resources": {
                "type": "array",
                "items": {
                  "type": "object",
                  "properties": {
                    "type": {
                      "type": "string",
                      "description": "Terraform resource type"
                    },
                    "name": {
                      "type": "string",
                      "description": "Resource name"
                    },
                    "file": {
                      "type": "string",
                      "description": "File containing the resource definition"
                    }
                  }
                }
              }
            }
          }
        },
        "required": ["analysis"]
      }
    },
    {
      "name": "get_terraform_plan",
      "description": "Execute 'terraform plan' and return the output",
      "inputSchema": {
        "type": "object",
        "properties": {}
      },
      "outputSchema": {
        "type": "object",
        "properties": {
          "plan": {
            "type": "string",
            "description": "Terraform plan output"
          }
        },
        "required": ["plan"]
      }
    },
    {
      "name": "apply_terraform",
      "description": "Apply Terraform configuration (WARNING: This will make actual changes to your infrastructure)",
      "inputSchema": {
        "type": "object",
        "properties": {
          "auto_approve": {
            "type": "boolean",
            "description": "Whether to auto-approve changes without confirmation"
          }
        }
      },
      "outputSchema": {
        "type": "object",
        "properties": {
          "output": {
            "type": "string",
            "description": "Terraform apply output"
          }
        },
        "required": ["output"]
      }
    },
    {
      "name": "validate_terraform",
      "description": "Validate Terraform configuration files",
      "inputSchema": {
        "type": "object",
        "properties": {}
      },
      "outputSchema": {
        "type": "object",
        "properties": {
          "valid": {
            "type": "boolean",
            "description": "Whether the configuration is valid"
          },
          "message": {
            "type": "string",
            "description": "Validation message"
          }
        },
        "required": ["valid", "message"]
      }
    },
    {
      "name": "get_terraform_state",
      "description": "Get the current Terraform state",
      "inputSchema": {
        "type": "object",
        "properties": {}
      },
      "outputSchema": {
        "type": "object",
        "properties": {
          "state": {
            "type": "string",
            "description": "Terraform state output"
          }
        },
        "required": ["state"]
      }
    },
    {
      "name": "init_terraform",
      "description": "Initialize a Terraform project",
      "inputSchema": {
        "type": "object",
        "properties": {}
      },
      "outputSchema": {
        "type": "object",
        "properties": {
          "output": {
            "type": "string",
            "description": "Terraform init output"
          }
        },
        "required": ["output"]
      }
    },
    {
      "name": "set_terraform_directory",
      "description": "Change the current Terraform project directory",
      "inputSchema": {
        "type": "object",
        "properties": {
          "directory": {
            "type": "string",
            "description": "Path to the new Terraform project directory"
          }
        },
        "required": ["directory"]
      },
      "outputSchema": {
        "type": "object",
        "properties": {
          "success": {
            "type": "boolean",
            "description": "Whether the directory change was successful"
          },
          "directory": {
            "type": "string",
            "description": "The new Terraform project directory path"
          },
          "message": {
            "type": "string",
            "description": "Status message"
          }
        },
        "required": ["success", "directory", "message"]
      }
    }
  ]
}"#;

pub struct McpHandler<'a> {
    tfmcp: &'a mut TfMcp,
    initialized: bool,
}

impl<'a> McpHandler<'a> {
    pub fn new(tfmcp: &'a mut TfMcp) -> Self {
        Self {
            tfmcp,
            initialized: false,
        }
    }

    pub async fn launch_mcp(&mut self, transport: &StdioTransport) -> anyhow::Result<()> {
        let mut stream = transport.receive();

        logging::info("MCP stdio transport server started. Waiting for JSON messages on stdin...");
        logging::send_log_message(transport, logging::LogLevel::Info, "tfmcp server initialized and ready").await?;

        while let Some(msg_result) = stream.next().await {
            match msg_result {
                Ok(Message::Request {
                    id, method, params, ..
                }) => {
                    logging::log_both(
                        transport,
                        logging::LogLevel::Debug,
                        &format!("Got Request: id={}, method={}, params={:?}", id, method, params)
                    ).await?;

                    // Handle initialization request first
                    if method == "initialize" {
                        if let Err(err) = self.handle_initialize(transport, id).await {
                            logging::error(&format!("Error handling initialize request: {}", err));
                        }
                        self.initialized = true;
                        continue;
                    }

                    // For all other requests, ensure we're initialized
                    if !self.initialized {
                        self.send_error_response(
                            transport,
                            id,
                            JsonRpcErrorCode::InvalidRequest,
                            "Server not initialized. Send 'initialize' request first.".to_string(),
                        )
                        .await?;
                        continue;
                    }

                    if let Err(err) = self.handle_request(transport, id, method, params).await {
                        logging::error(&format!("Error handling request: {:?}", err));
                        self.send_error_response(
                            transport,
                            id,
                            JsonRpcErrorCode::InternalError,
                            format!("Failed to handle request: {}", err),
                        )
                        .await?;
                    }
                }
                Ok(Message::Notification { method, params, .. }) => {
                    logging::log_both(
                        transport,
                        logging::LogLevel::Debug,
                        &format!("Got Notification: method={}, params={:?}", method, params)
                    ).await?;
                }
                Ok(Message::Response {
                    id, result, error, ..
                }) => {
                    logging::log_both(
                        transport,
                        logging::LogLevel::Debug,
                        &format!("Got Response: id={}, result={:?}, error={:?}", id, result, error)
                    ).await?;
                }
                Err(e) => {
                    logging::error(&format!("Error receiving message: {:?}", e));
                }
            }
        }

        Ok(())
    }

    async fn handle_request(
        &mut self,
        transport: &StdioTransport,
        id: u64,
        method: String,
        params: Option<serde_json::Value>,
    ) -> anyhow::Result<()> {
        match &*method {
            "initialize" => self.handle_initialize(transport, id).await?,
            "tools/list" => self.handle_tools_list(transport, id).await?,
            "tools/call" => {
                if let Some(params_val) = params {
                    self.handle_tools_call(transport, id, params_val).await?;
                }
            }
            "resources/list" => self.handle_resources_list(transport, id).await?,
            "prompts/list" => self.handle_prompts_list(transport, id).await?,
            _ => {
                self.send_error_response(
                    transport,
                    id,
                    JsonRpcErrorCode::MethodNotFound,
                    format!("Method not found: {}", method),
                )
                .await?;
            }
        }
        Ok(())
    }

    async fn handle_initialize(&self, transport: &StdioTransport, id: u64) -> anyhow::Result<()> {
        logging::info("Handling initialize request");
        
        // Create a properly structured capabilities response
        let response = Message::Response {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "capabilities": {
                    "experimental": {},
                    "prompts": { "listChanged": false },
                    "resources": { "listChanged": false, "subscribe": false },
                    "tools": { "listChanged": false }
                },
                "protocolVersion": "2024-11-05",
                "serverInfo": {
                    "name": "tfmcp",
                    "version": "0.1.0"
                }
            })),
            error: None,
        };
        
        // Log the response for debugging
        if let Ok(json_str) = serde_json::to_string_pretty(&response) {
            logging::debug(&format!("Sending initialize response: {}", json_str));
        }
        
        // Send the response
        match transport.send(response).await {
            Ok(_) => {
                logging::info("Initialize response sent successfully");
                Ok(())
            },
            Err(e) => {
                logging::error(&format!("Failed to send initialize response: {}", e));
                Err(anyhow::anyhow!("Failed to send initialize response: {}", e))
            }
        }
    }

    async fn handle_tools_list(&self, transport: &StdioTransport, id: u64) -> anyhow::Result<()> {
        let tools_value: serde_json::Value =
            serde_json::from_str(TOOLS_JSON).expect("tools.json must be valid JSON");

        let response = Message::Response {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(tools_value),
            error: None,
        };

        transport.send(response).await?;
        Ok(())
    }

    async fn handle_tools_call(
        &mut self,
        transport: &StdioTransport,
        id: u64,
        params_val: serde_json::Value,
    ) -> anyhow::Result<()> {
        let name = params_val
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        logging::info(&format!("Handling tools/call for tool: {}", name));

        match name {
            "list_terraform_resources" => {
                self.handle_list_terraform_resources(transport, id).await?;
            }
            "analyze_terraform" => {
                self.handle_analyze_terraform(transport, id, &params_val).await?;
            }
            "get_terraform_plan" => {
                self.handle_get_terraform_plan(transport, id).await?;
            }
            "apply_terraform" => {
                self.handle_apply_terraform(transport, id, &params_val).await?;
            }
            "validate_terraform" => {
                self.handle_validate_terraform(transport, id).await?;
            }
            "get_terraform_state" => {
                self.handle_get_terraform_state(transport, id).await?;
            }
            "init_terraform" => {
                self.handle_init_terraform(transport, id).await?;
            }
            "set_terraform_directory" => {
                self.handle_set_terraform_directory(transport, id, &params_val).await?;
            }
            _ => {
                self.send_error_response(
                    transport,
                    id,
                    JsonRpcErrorCode::MethodNotFound,
                    format!("Tool not found: {}", name),
                )
                .await?;
            }
        }

        Ok(())
    }

    async fn handle_list_terraform_resources(
        &self,
        transport: &StdioTransport,
        id: u64,
    ) -> anyhow::Result<()> {
        match self.tfmcp.list_resources().await {
            Ok(resources) => {
                let result_json = json!({ "resources": resources });
                let obj_as_str = serde_json::to_string(&result_json)?;
                self.send_text_response(transport, id, &obj_as_str).await?;
            }
            Err(err) => {
                self.send_error_response(
                    transport,
                    id,
                    JsonRpcErrorCode::InternalError,
                    format!("Failed to list Terraform resources: {}", err),
                )
                .await?;
            }
        }

        Ok(())
    }

    async fn handle_analyze_terraform(
        &mut self,
        transport: &StdioTransport,
        id: u64,
        params_val: &serde_json::Value,
    ) -> anyhow::Result<()> {
        // Get optional path parameter
        let path = params_val
            .pointer("/arguments/path")
            .and_then(Value::as_str)
            .map(PathBuf::from);

        // Analyze Terraform configurations
        match self.tfmcp.analyze_terraform().await {
            Ok(analysis) => {
                let result_json = json!({ "analysis": analysis });
                let obj_as_str = serde_json::to_string(&result_json)?;
                self.send_text_response(transport, id, &obj_as_str).await?;
            }
            Err(err) => {
                self.send_error_response(
                    transport,
                    id,
                    JsonRpcErrorCode::InternalError,
                    format!("Failed to analyze Terraform configuration: {}", err),
                )
                .await?;
            }
        }

        Ok(())
    }

    async fn handle_get_terraform_plan(
        &self,
        transport: &StdioTransport,
        id: u64,
    ) -> anyhow::Result<()> {
        match self.tfmcp.get_terraform_plan().await {
            Ok(plan) => {
                let result_json = json!({ "plan": plan });
                let obj_as_str = serde_json::to_string(&result_json)?;
                self.send_text_response(transport, id, &obj_as_str).await?;
            }
            Err(err) => {
                self.send_error_response(
                    transport,
                    id,
                    JsonRpcErrorCode::InternalError,
                    format!("Failed to get Terraform plan: {}", err),
                )
                .await?;
            }
        }

        Ok(())
    }

    async fn handle_apply_terraform(
        &self,
        transport: &StdioTransport,
        id: u64,
        params_val: &serde_json::Value,
    ) -> anyhow::Result<()> {
        let auto_approve = params_val
            .pointer("/arguments/auto_approve")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        match self.tfmcp.apply_terraform(auto_approve).await {
            Ok(result) => {
                let result_json = json!({ "result": result });
                let obj_as_str = serde_json::to_string(&result_json)?;
                self.send_text_response(transport, id, &obj_as_str).await?;
            }
            Err(err) => {
                self.send_error_response(
                    transport,
                    id,
                    JsonRpcErrorCode::InternalError,
                    format!("Failed to apply Terraform configuration: {}", err),
                )
                .await?;
            }
        }

        Ok(())
    }

    async fn handle_validate_terraform(
        &self,
        transport: &StdioTransport,
        id: u64,
    ) -> anyhow::Result<()> {
        match self.tfmcp.validate_configuration().await {
            Ok(result) => {
                // If validation succeeded, result will contain a success message
                let valid = !result.contains("Error:");
                let result_json = json!({
                    "valid": valid,
                    "message": result
                });
                let obj_as_str = serde_json::to_string(&result_json)?;
                self.send_text_response(transport, id, &obj_as_str).await?;
            }
            Err(err) => {
                self.send_error_response(
                    transport,
                    id,
                    JsonRpcErrorCode::InternalError,
                    format!("Failed to validate Terraform configuration: {}", err),
                )
                .await?;
            }
        }

        Ok(())
    }

    async fn handle_get_terraform_state(
        &self,
        transport: &StdioTransport,
        id: u64,
    ) -> anyhow::Result<()> {
        match self.tfmcp.get_state().await {
            Ok(state) => {
                let result_json = json!({ "state": state });
                let obj_as_str = serde_json::to_string(&result_json)?;
                self.send_text_response(transport, id, &obj_as_str).await?;
            }
            Err(err) => {
                self.send_error_response(
                    transport,
                    id,
                    JsonRpcErrorCode::InternalError,
                    format!("Failed to get Terraform state: {}", err),
                )
                .await?;
            }
        }

        Ok(())
    }

    async fn handle_init_terraform(
        &self,
        transport: &StdioTransport,
        id: u64,
    ) -> anyhow::Result<()> {
        match self.tfmcp.init_terraform().await {
            Ok(result) => {
                let result_json = json!({ "result": result });
                let obj_as_str = serde_json::to_string(&result_json)?;
                self.send_text_response(transport, id, &obj_as_str).await?;
            }
            Err(err) => {
                self.send_error_response(
                    transport,
                    id,
                    JsonRpcErrorCode::InternalError,
                    format!("Failed to initialize Terraform: {}", err),
                )
                .await?;
            }
        }

        Ok(())
    }

    async fn handle_resources_list(&self, transport: &StdioTransport, id: u64) -> anyhow::Result<()> {
        logging::info("Handling resources/list request");
        
        // Create a response with an empty resources list
        let response = Message::Response {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "resources": []
            })),
            error: None,
        };
        
        // Log the response for debugging
        if let Ok(json_str) = serde_json::to_string_pretty(&response) {
            logging::debug(&format!("Sending resources/list response: {}", json_str));
        }
        
        // Send the response
        match transport.send(response).await {
            Ok(_) => {
                logging::info("Resources list response sent successfully");
                Ok(())
            },
            Err(e) => {
                logging::error(&format!("Failed to send resources/list response: {}", e));
                Err(e.into())
            }
        }
    }

    async fn handle_prompts_list(&self, transport: &StdioTransport, id: u64) -> anyhow::Result<()> {
        logging::info("Handling prompts/list request");
        
        // Create a response with an empty prompts list
        let response = Message::Response {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "prompts": []
            })),
            error: None,
        };
        
        // Log the response for debugging
        if let Ok(json_str) = serde_json::to_string_pretty(&response) {
            logging::debug(&format!("Sending prompts/list response: {}", json_str));
        }
        
        // Send the response
        match transport.send(response).await {
            Ok(_) => {
                logging::info("Prompts list response sent successfully");
                Ok(())
            },
            Err(e) => {
                logging::error(&format!("Failed to send prompts/list response: {}", e));
                Err(e.into())
            }
        }
    }

    async fn send_text_response(
        &self,
        transport: &StdioTransport,
        id: u64,
        text: &str,
    ) -> anyhow::Result<()> {
        logging::info(&format!("Sending text response for id {}", id));
        
        // Create a properly structured text response
        let response = Message::Response {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "content": [{
                    "type": "text",
                    "text": text
                }]
            })),
            error: None,
        };
        
        // Log the response for debugging
        if let Ok(json_str) = serde_json::to_string_pretty(&response) {
            logging::debug(&format!("Sending text response: {}", json_str));
        }
        
        // Send the response
        match transport.send(response).await {
            Ok(_) => {
                logging::info("Text response sent successfully");
                Ok(())
            },
            Err(e) => {
                logging::error(&format!("Failed to send text response: {}", e));
                Err(anyhow::anyhow!("Failed to send text response: {}", e))
            }
        }
    }

    async fn send_error_response(
        &self,
        transport: &StdioTransport,
        id: u64,
        code: JsonRpcErrorCode,
        message: String,
    ) -> anyhow::Result<()> {
        logging::warn(&format!("Sending error response for id {}: {}", id, message));
        
        // Create a properly structured error response
        let response = Message::Response {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(json!({
                "code": code as i32,
                "message": message
            })),
        };
        
        // Log the response for debugging
        if let Ok(json_str) = serde_json::to_string_pretty(&response) {
            logging::debug(&format!("Sending error response: {}", json_str));
        }
        
        // Send the response
        match transport.send(response).await {
            Ok(_) => {
                logging::info("Error response sent successfully");
                Ok(())
            },
            Err(e) => {
                logging::error(&format!("Failed to send error response: {}", e));
                Err(anyhow::anyhow!("Failed to send error response: {}", e))
            }
        }
    }

    // 新しいハンドラー: Terraformディレクトリを変更する
    async fn handle_set_terraform_directory(
        &mut self,
        transport: &StdioTransport,
        id: u64,
        params_val: &serde_json::Value,
    ) -> anyhow::Result<()> {
        logging::info("Handling set_terraform_directory request");
        
        // パラメータから新しいディレクトリパスを取得
        let directory = match params_val.get("directory").and_then(|v| v.as_str()) {
            Some(dir) => dir.to_string(),
            None => {
                return self.send_error_response(
                    transport,
                    id,
                    JsonRpcErrorCode::InvalidParams,
                    "Missing required parameter: directory".to_string(),
                ).await;
            }
        };
        
        // ディレクトリを変更
        match self.tfmcp.change_project_directory(directory) {
            Ok(()) => {
                // 現在のディレクトリを取得して応答
                let current_dir = self.tfmcp.get_project_directory();
                let current_dir_str = current_dir.to_string_lossy().to_string();
                
                let response = Message::Response {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(json!({
                        "success": true,
                        "directory": current_dir_str,
                        "message": format!("Successfully changed to Terraform project directory: {}", current_dir_str)
                    })),
                    error: None,
                };
                
                // レスポンスをログに記録
                if let Ok(json_str) = serde_json::to_string_pretty(&response) {
                    logging::debug(&format!("Sending set_terraform_directory response: {}", json_str));
                }
                
                // レスポンスを送信
                match transport.send(response).await {
                    Ok(_) => {
                        logging::info("Set terraform directory response sent successfully");
                        Ok(())
                    },
                    Err(e) => {
                        logging::error(&format!("Failed to send set_terraform_directory response: {}", e));
                        Err(e.into())
                    }
                }
            },
            Err(e) => {
                self.send_error_response(
                    transport,
                    id,
                    JsonRpcErrorCode::InternalError,
                    format!("Failed to change Terraform directory: {}", e),
                ).await
            }
        }
    }
}
