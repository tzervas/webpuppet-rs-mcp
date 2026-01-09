//! Tool definitions and registry for MCP server.

use std::collections::HashMap;
use std::sync::Arc;

use serde::Deserialize;
use serde_json::json;
use tokio::sync::RwLock;

use embeddenator_webpuppet::{
    BrowserDetector, InterventionHandler, InterventionState, Operation, PermissionGuard,
    PromptRequest, Provider, ScreeningConfig, WebPuppet,
};

use crate::error::{Error, Result};
use crate::protocol::{ContentItem, ToolCallResult, ToolDefinition};

/// Tool trait for implementing MCP tools.
#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    /// Get the tool definition.
    fn definition(&self) -> ToolDefinition;

    /// Execute the tool with the given arguments.
    async fn execute(
        &self,
        arguments: serde_json::Value,
        context: &ToolContext,
    ) -> Result<ToolCallResult>;
}

/// Context passed to tools during execution.
pub struct ToolContext {
    /// WebPuppet instance (lazy-initialized).
    pub puppet: Arc<RwLock<Option<WebPuppet>>>,
    /// Permission guard.
    pub permissions: Arc<PermissionGuard>,
    /// Screening configuration.
    pub screening_config: ScreeningConfig,
    /// Intervention handler for human-in-the-loop.
    pub intervention_handler: Arc<RwLock<InterventionHandler>>,
    /// Whether to run browser in headless mode (default: true).
    pub headless: bool,
}

impl ToolContext {
    /// Create a new tool context.
    pub fn new(permissions: PermissionGuard) -> Self {
        Self {
            puppet: Arc::new(RwLock::new(None)),
            permissions: Arc::new(permissions),
            screening_config: ScreeningConfig::default(),
            intervention_handler: Arc::new(RwLock::new(InterventionHandler::new())),
            headless: true,
        }
    }

    /// Create a new tool context with visible browser (non-headless).
    pub fn with_visible_browser(permissions: PermissionGuard) -> Self {
        Self {
            puppet: Arc::new(RwLock::new(None)),
            permissions: Arc::new(permissions),
            screening_config: ScreeningConfig::default(),
            intervention_handler: Arc::new(RwLock::new(InterventionHandler::new())),
            headless: false,
        }
    }

    /// Get or create the WebPuppet instance.
    pub async fn get_puppet(&self) -> Result<WebPuppet> {
        let guard = self.puppet.read().await;
        if let Some(ref _puppet) = *guard {
            // Clone isn't implemented, so we'll need to recreate
            drop(guard);
        } else {
            drop(guard);
        }

        // Create new puppet with headless setting
        let puppet = WebPuppet::builder()
            .with_all_providers()
            .headless(self.headless)
            .with_screening_config(self.screening_config.clone())
            .build()
            .await?;

        Ok(puppet)
    }
}

/// Registry of available tools.
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
    context: Arc<ToolContext>,
}

impl ToolRegistry {
    /// Create a new tool registry with default tools (headless browser).
    pub fn new(permissions: PermissionGuard) -> Self {
        Self::with_context(ToolContext::new(permissions))
    }

    /// Create a new tool registry with visible browser.
    pub fn with_visible_browser(permissions: PermissionGuard) -> Self {
        Self::with_context(ToolContext::with_visible_browser(permissions))
    }

    /// Create a new tool registry with custom context.
    fn with_context(context: ToolContext) -> Self {
        let context = Arc::new(context);
        let mut tools: HashMap<String, Arc<dyn Tool>> = HashMap::new();

        // Register built-in tools
        let prompt_tool = Arc::new(PromptTool);
        tools.insert(prompt_tool.definition().name.clone(), prompt_tool);

        let list_providers_tool = Arc::new(ListProvidersTool);
        tools.insert(
            list_providers_tool.definition().name.clone(),
            list_providers_tool,
        );

        let provider_caps_tool = Arc::new(ProviderCapabilitiesTool);
        tools.insert(
            provider_caps_tool.definition().name.clone(),
            provider_caps_tool,
        );

        let detect_browsers_tool = Arc::new(DetectBrowsersTool);
        tools.insert(
            detect_browsers_tool.definition().name.clone(),
            detect_browsers_tool,
        );

        let screenshot_tool = Arc::new(ScreenshotTool);
        tools.insert(screenshot_tool.definition().name.clone(), screenshot_tool);

        let check_permission_tool = Arc::new(CheckPermissionTool);
        tools.insert(
            check_permission_tool.definition().name.clone(),
            check_permission_tool,
        );

        // Intervention tools
        let intervention_status_tool = Arc::new(InterventionStatusTool);
        tools.insert(
            intervention_status_tool.definition().name.clone(),
            intervention_status_tool,
        );

        let intervention_complete_tool = Arc::new(InterventionCompleteTool);
        tools.insert(
            intervention_complete_tool.definition().name.clone(),
            intervention_complete_tool,
        );

        let intervention_pause_tool = Arc::new(InterventionPauseTool);
        tools.insert(
            intervention_pause_tool.definition().name.clone(),
            intervention_pause_tool,
        );

        let intervention_resume_tool = Arc::new(InterventionResumeTool);
        tools.insert(
            intervention_resume_tool.definition().name.clone(),
            intervention_resume_tool,
        );

        // Navigation and status tools
        let navigate_tool = Arc::new(NavigateTool);
        tools.insert(navigate_tool.definition().name.clone(), navigate_tool);

        let browser_status_tool = Arc::new(BrowserStatusTool);
        tools.insert(
            browser_status_tool.definition().name.clone(),
            browser_status_tool,
        );

        Self { tools, context }
    }

    /// Get tool definitions.
    pub fn list_tools(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|t| t.definition()).collect()
    }

    /// Execute a tool by name.
    pub async fn execute(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<ToolCallResult> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| Error::ToolNotFound(name.to_string()))?;

        tool.execute(arguments, &self.context).await
    }

    /// Register a custom tool.
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        let name = tool.definition().name.clone();
        self.tools.insert(name, tool);
    }
}

// ============================================================================
// Built-in Tools
// ============================================================================

/// Tool for sending prompts to AI providers.
pub struct PromptTool;

#[derive(Debug, Deserialize)]
struct PromptArgs {
    /// Provider to use (claude, grok, gemini).
    provider: String,
    /// Message to send.
    message: String,
    /// Optional context/system prompt.
    context: Option<String>,
}

#[async_trait::async_trait]
impl Tool for PromptTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "webpuppet_prompt".into(),
            description: "Send a prompt through browser automation (AI providers + select web tools). Uses existing authenticated sessions.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "provider": {
                        "type": "string",
                        "enum": ["claude", "grok", "gemini", "chatgpt", "perplexity", "notebooklm", "kaggle"],
                        "description": "Provider/tool to use"
                    },
                    "message": {
                        "type": "string",
                        "description": "The prompt message to send"
                    },
                    "context": {
                        "type": "string",
                        "description": "Optional context or system instructions"
                    }
                },
                "required": ["provider", "message"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: serde_json::Value,
        context: &ToolContext,
    ) -> Result<ToolCallResult> {
        // Check permission
        context
            .permissions
            .require(Operation::SendPrompt)
            .map_err(|e| Error::PermissionDenied(e.to_string()))?;

        // Parse arguments
        let args: PromptArgs =
            serde_json::from_value(arguments).map_err(|e| Error::InvalidParams(e.to_string()))?;

        // Parse provider
        let provider = match args.provider.to_lowercase().as_str() {
            "claude" => Provider::Claude,
            "grok" => Provider::Grok,
            "gemini" => Provider::Gemini,
            "chatgpt" | "openai" => Provider::ChatGpt,
            "perplexity" => Provider::Perplexity,
            "notebooklm" | "notebook" => Provider::NotebookLm,
            "kaggle" => Provider::Kaggle,
            _ => {
                return Err(Error::InvalidParams(format!(
                    "unknown provider: {}",
                    args.provider
                )))
            }
        };

        // Build request
        let mut request = PromptRequest::new(args.message);
        if let Some(ctx) = args.context {
            request = request.with_context(ctx);
        }

        // Get puppet and send prompt
        let puppet = context.get_puppet().await?;

        // Authenticate if needed
        puppet.authenticate(provider).await?;

        // Send with screening
        let (response, screening) = puppet.prompt_screened(provider, request).await?;

        // Close puppet
        puppet.close().await.ok();

        // Format result
        let result_text = if screening.passed {
            response.text
        } else {
            format!(
                "[SECURITY WARNING: Response had risk score {:.2}]\n\n{}",
                screening.risk_score, response.text
            )
        };

        Ok(ToolCallResult {
            content: vec![ContentItem::text(result_text)],
            is_error: false,
        })
    }
}

/// Tool for listing available AI providers.
pub struct ListProvidersTool;

#[async_trait::async_trait]
impl Tool for ListProvidersTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "webpuppet_list_providers".into(),
            description: "List available AI providers and their status.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn execute(
        &self,
        _arguments: serde_json::Value,
        _context: &ToolContext,
    ) -> Result<ToolCallResult> {
        let providers = [
            (
                "claude",
                "Claude (Anthropic)",
                "https://claude.ai",
                "Large context, artifacts, code",
            ),
            (
                "grok",
                "Grok (X/xAI)",
                "https://x.com/i/grok",
                "Real-time info, integrated with X",
            ),
            (
                "gemini",
                "Gemini (Google)",
                "https://gemini.google.com",
                "Google integration, large context",
            ),
            (
                "chatgpt",
                "ChatGPT (OpenAI)",
                "https://chat.openai.com",
                "GPT-4o, vision, code, web search",
            ),
            (
                "perplexity",
                "Perplexity AI",
                "https://www.perplexity.ai",
                "Search-focused, sources cited",
            ),
            (
                "notebooklm",
                "NotebookLM (Google)",
                "https://notebooklm.google.com",
                "Research assistant, 500k context",
            ),
            (
                "kaggle",
                "Kaggle (Datasets)",
                "https://www.kaggle.com/datasets",
                "Dataset search/catalog; returns dataset page links",
            ),
        ];

        let text = providers
            .iter()
            .map(|(id, name, url, features)| {
                format!(
                    "- **{}** (`{}`): [{}]({})\n  _{}_",
                    name, id, url, url, features
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        Ok(ToolCallResult {
            content: vec![ContentItem::text(format!(
                "# Available Providers\n\n{}\n\n*Note: Uses browser sessions; some providers require login.*",
                text
            ))],
            is_error: false,
        })
    }
}

/// Tool for retrieving declared provider capabilities.
pub struct ProviderCapabilitiesTool;

#[derive(Debug, Deserialize)]
struct ProviderCapabilitiesArgs {
    /// Provider/tool to inspect.
    provider: String,
}

#[async_trait::async_trait]
impl Tool for ProviderCapabilitiesTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "webpuppet_provider_capabilities".into(),
            description: "Get declared capabilities for a provider/tool (conversation, vision, file upload, web search, etc).".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "provider": {
                        "type": "string",
                        "enum": ["claude", "grok", "gemini", "chatgpt", "perplexity", "notebooklm", "kaggle"],
                        "description": "Provider/tool to inspect"
                    }
                },
                "required": ["provider"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: serde_json::Value,
        context: &ToolContext,
    ) -> Result<ToolCallResult> {
        context
            .permissions
            .require(Operation::ReadContent)
            .map_err(|e| Error::PermissionDenied(e.to_string()))?;

        let args: ProviderCapabilitiesArgs =
            serde_json::from_value(arguments).map_err(|e| Error::InvalidParams(e.to_string()))?;

        let provider = match args.provider.to_lowercase().as_str() {
            "claude" => Provider::Claude,
            "grok" => Provider::Grok,
            "gemini" => Provider::Gemini,
            "chatgpt" | "openai" => Provider::ChatGpt,
            "perplexity" => Provider::Perplexity,
            "notebooklm" | "notebook" => Provider::NotebookLm,
            "kaggle" => Provider::Kaggle,
            _ => {
                return Err(Error::InvalidParams(format!(
                    "unknown provider: {}",
                    args.provider
                )))
            }
        };

        // Build a puppet (no auth needed just to query static capabilities).
        let puppet = context.get_puppet().await?;

        let caps = puppet
            .provider_capabilities(provider)
            .ok_or_else(|| Error::InvalidParams(format!("provider not available: {}", provider)))?;

        puppet.close().await.ok();

        Ok(ToolCallResult {
            content: vec![ContentItem::text(
                serde_json::to_string_pretty(&json!({
                    "provider": provider.to_string(),
                    "capabilities": {
                        "conversation": caps.conversation,
                        "vision": caps.vision,
                        "file_upload": caps.file_upload,
                        "code_execution": caps.code_execution,
                        "web_search": caps.web_search,
                        "max_context": caps.max_context,
                        "models": caps.models,
                        "note": "Declared capabilities (not runtime UI detection)."
                    }
                }))
                .map_err(|e| Error::Internal(e.to_string()))?,
            )],
            is_error: false,
        })
    }
}

/// Tool for detecting installed browsers.
pub struct DetectBrowsersTool;

#[async_trait::async_trait]
impl Tool for DetectBrowsersTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "webpuppet_detect_browsers".into(),
            description: "Detect installed browsers that can be used for automation.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn execute(
        &self,
        _arguments: serde_json::Value,
        _context: &ToolContext,
    ) -> Result<ToolCallResult> {
        let browsers = BrowserDetector::detect_all();

        if browsers.is_empty() {
            return Ok(ToolCallResult {
                content: vec![ContentItem::text(
                    "No supported browsers detected. Please install Brave, Chrome, or Chromium.",
                )],
                is_error: true,
            });
        }

        let text = browsers
            .iter()
            .map(|b| {
                let version = b.version.as_deref().unwrap_or("unknown");
                let profiles = b.list_profiles().unwrap_or_default();
                format!(
                    "- **{}** ({})\n  - Path: `{}`\n  - Data: `{}`\n  - Profiles: {}",
                    b.browser_type,
                    version,
                    b.executable_path.display(),
                    b.user_data_dir.display(),
                    if profiles.is_empty() {
                        "none".to_string()
                    } else {
                        profiles.join(", ")
                    }
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        Ok(ToolCallResult {
            content: vec![ContentItem::text(format!(
                "# Detected Browsers\n\n{}",
                text
            ))],
            is_error: false,
        })
    }
}

/// Tool for taking screenshots.
pub struct ScreenshotTool;

#[derive(Debug, Deserialize)]
struct ScreenshotArgs {
    /// URL to screenshot.
    url: String,
}

#[async_trait::async_trait]
impl Tool for ScreenshotTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "webpuppet_screenshot".into(),
            description: "Take a screenshot of a web page. Only allowed domains can be accessed."
                .into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "URL to take a screenshot of"
                    }
                },
                "required": ["url"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: serde_json::Value,
        context: &ToolContext,
    ) -> Result<ToolCallResult> {
        let args: ScreenshotArgs =
            serde_json::from_value(arguments).map_err(|e| Error::InvalidParams(e.to_string()))?;

        // Check permissions for this URL
        context
            .permissions
            .require_with_url(Operation::Navigate, &args.url)
            .map_err(|e| Error::PermissionDenied(e.to_string()))?;

        context
            .permissions
            .require(Operation::Screenshot)
            .map_err(|e| Error::PermissionDenied(e.to_string()))?;

        // For now, return a placeholder since actual screenshot requires full browser impl
        Ok(ToolCallResult {
            content: vec![ContentItem::text(format!(
                "Screenshot of `{}` would be captured here.\n\n*Note: Full browser implementation required for actual screenshots.*",
                args.url
            ))],
            is_error: false,
        })
    }
}

/// Tool for checking permissions.
pub struct CheckPermissionTool;

#[derive(Debug, Deserialize)]
struct CheckPermissionArgs {
    /// Operation to check.
    operation: String,
    /// Optional URL context.
    url: Option<String>,
}

#[async_trait::async_trait]
impl Tool for CheckPermissionTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "webpuppet_check_permission".into(),
            description: "Check if an operation is allowed by the security policy.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "description": "Operation to check (e.g., Navigate, SendPrompt, DeleteAccount)"
                    },
                    "url": {
                        "type": "string",
                        "description": "Optional URL context for navigation checks"
                    }
                },
                "required": ["operation"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: serde_json::Value,
        context: &ToolContext,
    ) -> Result<ToolCallResult> {
        let args: CheckPermissionArgs =
            serde_json::from_value(arguments).map_err(|e| Error::InvalidParams(e.to_string()))?;

        // Map string to Operation
        let operation = match args.operation.to_lowercase().as_str() {
            "navigate" => Operation::Navigate,
            "sendprompt" | "send_prompt" => Operation::SendPrompt,
            "readresponse" | "read_response" => Operation::ReadResponse,
            "screenshot" => Operation::Screenshot,
            "click" => Operation::Click,
            "typetext" | "type_text" => Operation::TypeText,
            "deleteaccount" | "delete_account" => Operation::DeleteAccount,
            "changepassword" | "change_password" => Operation::ChangePassword,
            _ => {
                return Ok(ToolCallResult {
                    content: vec![ContentItem::text(format!(
                        "Unknown operation: `{}`\n\nValid operations: Navigate, SendPrompt, ReadResponse, Screenshot, Click, TypeText, DeleteAccount, ChangePassword",
                        args.operation
                    ))],
                    is_error: true,
                });
            }
        };

        let decision = if let Some(url) = args.url {
            context.permissions.check_with_url(operation, &url)
        } else {
            context.permissions.check(operation)
        };

        let status = if decision.allowed {
            "✅ ALLOWED"
        } else {
            "❌ DENIED"
        };
        let text = format!(
            "# Permission Check\n\n**Operation**: `{}`\n**Status**: {}\n**Reason**: {}\n**Risk Level**: {}/10",
            operation, status, decision.reason, decision.risk_level
        );

        Ok(ToolCallResult {
            content: vec![ContentItem::text(text)],
            is_error: false,
        })
    }
}

// ============================================================================
// Intervention Tools
// ============================================================================

/// Tool for checking intervention status.
pub struct InterventionStatusTool;

#[async_trait::async_trait]
impl Tool for InterventionStatusTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "webpuppet_intervention_status".into(),
            description: "Check if human intervention is needed (captcha, 2FA, etc.). Returns current automation state and any pending intervention reason.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn execute(
        &self,
        _arguments: serde_json::Value,
        context: &ToolContext,
    ) -> Result<ToolCallResult> {
        let handler = context.intervention_handler.read().await;
        let state = handler.state();
        let reason = handler.current_reason();

        let state_str = match state {
            InterventionState::Running => "🟢 Running",
            InterventionState::WaitingForHuman => "🟡 Waiting for human",
            InterventionState::Resuming => "🔵 Resuming",
            InterventionState::TimedOut => "🔴 Timed out",
            InterventionState::Cancelled => "⚫ Cancelled",
        };

        let text = if let Some(reason) = reason {
            format!(
                "# Intervention Status\n\n**State**: {}\n**Reason**: {}\n\n⚠️ **Action Required**: Please complete the intervention in the browser, then call `webpuppet_intervention_complete` with success=true.",
                state_str, reason
            )
        } else {
            format!(
                "# Intervention Status\n\n**State**: {}\n\nNo intervention currently required. Automation is running normally.",
                state_str
            )
        };

        Ok(ToolCallResult {
            content: vec![ContentItem::text(text)],
            is_error: false,
        })
    }
}

/// Tool for signaling intervention completion.
pub struct InterventionCompleteTool;

#[derive(Debug, Deserialize)]
struct InterventionCompleteArgs {
    /// Whether the intervention was successful.
    success: bool,
    /// Optional message about the intervention.
    message: Option<String>,
}

#[async_trait::async_trait]
impl Tool for InterventionCompleteTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "webpuppet_intervention_complete".into(),
            description: "Signal that a human intervention (captcha, 2FA, etc.) has been completed. Call this after manually handling the intervention in the browser.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "success": {
                        "type": "boolean",
                        "description": "Whether the intervention was completed successfully"
                    },
                    "message": {
                        "type": "string",
                        "description": "Optional message about what was done"
                    }
                },
                "required": ["success"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: serde_json::Value,
        context: &ToolContext,
    ) -> Result<ToolCallResult> {
        let args: InterventionCompleteArgs =
            serde_json::from_value(arguments).map_err(|e| Error::InvalidParams(e.to_string()))?;

        let handler = context.intervention_handler.read().await;
        handler.complete(args.success, args.message.clone());

        let status = if args.success {
            "✅ SUCCESS"
        } else {
            "❌ FAILED"
        };
        let text = format!(
            "# Intervention Complete\n\n**Status**: {}\n**Message**: {}\n\nAutomation will now resume.",
            status,
            args.message.unwrap_or_else(|| "None".into())
        );

        Ok(ToolCallResult {
            content: vec![ContentItem::text(text)],
            is_error: false,
        })
    }
}

/// Tool for pausing automation.
pub struct InterventionPauseTool;

#[async_trait::async_trait]
impl Tool for InterventionPauseTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "webpuppet_pause".into(),
            description: "Pause browser automation. Use this when you need to manually interact with the browser.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn execute(
        &self,
        _arguments: serde_json::Value,
        context: &ToolContext,
    ) -> Result<ToolCallResult> {
        let handler = context.intervention_handler.read().await;
        handler.pause();

        Ok(ToolCallResult {
            content: vec![ContentItem::text(
                "# Automation Paused\n\n⏸️ Automation is now paused. The browser is available for manual interaction.\n\nCall `webpuppet_resume` when ready to continue."
            )],
            is_error: false,
        })
    }
}

/// Tool for resuming automation.
pub struct InterventionResumeTool;

#[async_trait::async_trait]
impl Tool for InterventionResumeTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "webpuppet_resume".into(),
            description: "Resume browser automation after a pause or manual intervention.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn execute(
        &self,
        _arguments: serde_json::Value,
        context: &ToolContext,
    ) -> Result<ToolCallResult> {
        let handler = context.intervention_handler.read().await;
        handler.resume();

        Ok(ToolCallResult {
            content: vec![ContentItem::text(
                "# Automation Resumed\n\n▶️ Automation has been resumed. Browser operations will continue."
            )],
            is_error: false,
        })
    }
}

/// Tool for navigating to a URL (for testing).
pub struct NavigateTool;

#[derive(Debug, Deserialize)]
struct NavigateArgs {
    /// URL to navigate to.
    url: String,
}

#[async_trait::async_trait]
impl Tool for NavigateTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "webpuppet_navigate".into(),
            description: "Navigate browser to a URL. Opens a browser window if not already open."
                .into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "URL to navigate to"
                    }
                },
                "required": ["url"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: serde_json::Value,
        context: &ToolContext,
    ) -> Result<ToolCallResult> {
        // Check permission
        context
            .permissions
            .require(Operation::Navigate)
            .map_err(|e| Error::PermissionDenied(e.to_string()))?;

        // Parse arguments
        let args: NavigateArgs =
            serde_json::from_value(arguments).map_err(|e| Error::InvalidParams(e.to_string()))?;

        // Get puppet and navigate
        let puppet = context.get_puppet().await?;

        // Get session (using Grok as default provider for navigation)
        let session = puppet.get_session(Provider::Grok).await?;

        // Navigate
        session.navigate(&args.url).await?;

        // Get current URL and title
        let current_url = session
            .current_url()
            .await
            .unwrap_or_else(|_| args.url.clone());
        let title = session
            .get_title()
            .await
            .unwrap_or_else(|_| "Unknown".into());

        Ok(ToolCallResult {
            content: vec![ContentItem::text(format!(
                "# Browser Navigated\n\n✅ Successfully navigated to URL.\n\n- **URL**: {}\n- **Title**: {}",
                current_url, title
            ))],
            is_error: false,
        })
    }
}

/// Tool for getting browser status.
pub struct BrowserStatusTool;

#[async_trait::async_trait]
impl Tool for BrowserStatusTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "webpuppet_browser_status".into(),
            description: "Get current browser status including URL, title, and visibility.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn execute(
        &self,
        _arguments: serde_json::Value,
        context: &ToolContext,
    ) -> Result<ToolCallResult> {
        let guard = context.puppet.read().await;

        if guard.is_none() {
            return Ok(ToolCallResult {
                content: vec![ContentItem::text(
                    "# Browser Status\n\n⚪ No browser session is currently active.\n\nA browser will be launched when you use `webpuppet_navigate` or `webpuppet_prompt`."
                )],
                is_error: false,
            });
        }

        // Return basic status
        let visibility = if context.headless {
            "Headless"
        } else {
            "Visible"
        };

        Ok(ToolCallResult {
            content: vec![ContentItem::text(format!(
                "# Browser Status\n\n🟢 Browser session is active.\n\n- **Mode**: {}\n- **Providers**: Grok, Claude, Gemini",
                visibility
            ))],
            is_error: false,
        })
    }
}

// We need async-trait
mod async_trait_impl {
    pub use async_trait::async_trait;
}
pub use async_trait_impl::async_trait;
