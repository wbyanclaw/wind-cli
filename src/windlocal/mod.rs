//! windlocal URI parse, schema validation, and action execution
//!
//! P0 grammar:
//!   windlocal://page?kind=<PageKind>&target=<workspace-relative-path>
//!   windlocal://command?id=<CommandId>
//!
//! P0: only parse/validate, do NOT execute external programs

use crate::errors::WindError;
use serde_json::Value as JsonValue;

// =============================================================================
// Action schema
// =============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum WindAction {
    Page { kind: PageKind, target: String },
    Command { id: CommandId },
}

#[derive(Debug, Clone, PartialEq)]
pub enum PageKind {
    File,
    Search,
    App,
    Settings,
}

impl PageKind {
    pub fn from_str(s: &str) -> Result<Self, WindError> {
        match s.to_lowercase().as_str() {
            "file" => Ok(Self::File),
            "search" => Ok(Self::Search),
            "app" => Ok(Self::App),
            "settings" => Ok(Self::Settings),
            other => Err(WindError::InvalidPageKind(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandId {
    ShowWorkspace,
    ShowSettings,
    CheckUpgrade,
}

impl CommandId {
    pub fn from_str(s: &str) -> Result<Self, WindError> {
        match s.to_lowercase().as_str() {
            "show_workspace" => Ok(Self::ShowWorkspace),
            "show_settings" => Ok(Self::ShowSettings),
            "check_upgrade" => Ok(Self::CheckUpgrade),
            other => Err(WindError::InvalidCommandId(other.to_string())),
        }
    }
}

// =============================================================================
// Parse
// =============================================================================

/// Parse a windlocal URI into a WindAction
pub fn parse(uri: &str) -> anyhow::Result<WindAction> {
    // Reject non-windlocal schemes
    if !uri.starts_with("windlocal://") {
        return Err(WindError::InvalidScheme(uri.split(':').next().unwrap_or("").to_string()).into());
    }

    let uri = uri
        .strip_prefix("windlocal://")
        .ok_or_else(|| WindError::InvalidScheme("windlocal".to_string()))?;

    // Must have action type: "page" or "command"
    let (action_type, params) = uri
        .split_once('?')
        .ok_or_else(|| WindError::MissingParam("action parameters".to_string()))?;

    match action_type {
        "page" => {
            reject_unknown_params(params, &["kind", "target"])?;
            Ok(WindAction::Page {
                kind: parse_page_action(params)?,
                target: parse_page_target(params)?,
            })
        }
        "command" => {
            reject_unknown_params(params, &["id"])?;
            Ok(WindAction::Command {
                id: parse_command_action(params)?,
            })
        }
        other => Err(WindError::InvalidActionType).map_err(Into::into),
    }
}

fn reject_unknown_params(params: &str, allowed: &[&str]) -> Result<(), WindError> {
    for pair in params.split('&') {
        let (key, _) = pair
            .split_once('=')
            .ok_or_else(|| WindError::MissingParam(pair.to_string()))?;
        if !allowed.contains(&key) {
            return Err(WindError::UnknownParam(key.to_string()));
        }
    }
    Ok(())
}

fn parse_page_action(params: &str) -> Result<PageKind, WindError> {
    for pair in params.split('&') {
        let (key, value) = pair.split_once('=').ok_or_else(|| WindError::MissingParam(pair.to_string()))?;
        if key == "kind" {
            let decoded = urlencoding::decode(value)
                .map_err(|_| WindError::Usage(format!("invalid URL encoding in kind: {}", value)))?
                .to_string();
            return PageKind::from_str(&decoded);
        }
    }
    Err(WindError::MissingParam("kind".to_string()))
}

fn parse_page_target(params: &str) -> Result<String, WindError> {
    for pair in params.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            if key == "target" {
                let decoded = urlencoding::decode(value)
                    .map_err(|_| WindError::Usage(format!("invalid URL encoding in target: {}", value)))?
                    .to_string();
                return Ok(decoded);
            }
        }
    }
    Err(WindError::MissingParam("target".to_string()))
}

fn parse_command_action(params: &str) -> Result<CommandId, WindError> {
    for pair in params.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            if key == "id" {
                let decoded = urlencoding::decode(value)
                    .map_err(|_| WindError::Usage(format!("invalid URL encoding in id: {}", value)))?
                    .to_string();
                return CommandId::from_str(&decoded);
            }
        }
    }
    Err(WindError::MissingParam("id".to_string()))
}

// =============================================================================
// Validate
// =============================================================================

/// Validate a parsed action (P0: only whitelist checks)
pub fn validate(action: &WindAction) -> anyhow::Result<()> {
    match action {
        WindAction::Page { target, .. } => {
            // target must not contain path traversal
            if target.contains("..") || target.starts_with('/') {
                return Err(WindError::ActionBlocked("path traversal in target".to_string()).into());
            }
            Ok(())
        }
        WindAction::Command { .. } => {
            // All P0 command IDs are non-destructive, already validated in parse
            Ok(())
        }
    }
}

// =============================================================================
// Output
// =============================================================================

/// Convert action to JSON for --json output
pub fn action_to_json(action: &WindAction) -> JsonValue {
    match action {
        WindAction::Page { kind, target } => serde_json::json!({
            "type": "page",
            "kind": kind_to_str(kind),
            "target": target
        }),
        WindAction::Command { id } => serde_json::json!({
            "type": "command",
            "id": command_id_to_str(id)
        }),
    }
}

fn kind_to_str(kind: &PageKind) -> &'static str {
    match kind {
        PageKind::File => "file",
        PageKind::Search => "search",
        PageKind::App => "app",
        PageKind::Settings => "settings",
    }
}

fn command_id_to_str(id: &CommandId) -> &'static str {
    match id {
        CommandId::ShowWorkspace => "show_workspace",
        CommandId::ShowSettings => "show_settings",
        CommandId::CheckUpgrade => "check_upgrade",
    }
}
