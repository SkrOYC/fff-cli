use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    Request(JsonRpcRequest),
    Response(JsonRpcResponse),
    Notification(JsonRpcNotification),
}

impl JsonRpcRequest {
    #[must_use]
    pub fn new(id: u64, method: impl Into<String>, params: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.into(),
            params,
        }
    }
}

impl JsonRpcResponse {
    #[must_use]
    pub fn success(id: u64, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(id),
            result: Some(result),
            error: None,
        }
    }

    #[must_use]
    pub fn error(id: u64, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(id),
            result: None,
            error: Some(error),
        }
    }

    #[must_use]
    pub fn parse_error(error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: None,
            result: None,
            error: Some(error),
        }
    }
}

impl JsonRpcNotification {
    #[must_use]
    pub fn new(method: impl Into<String>, params: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
        }
    }

    #[must_use]
    pub fn result(items: serde_json::Value) -> Self {
        Self::new("result", serde_json::json!({ "items": items }))
    }
}

impl JsonRpcError {
    #[must_use]
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::ParseError as i32,
            message: message.into(),
            data: None,
        }
    }

    #[must_use]
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::InvalidRequest as i32,
            message: message.into(),
            data: None,
        }
    }

    #[must_use]
    pub fn method_not_found(method: &str) -> Self {
        Self {
            code: ErrorCode::MethodNotFound as i32,
            message: format!("method not found: {method}"),
            data: None,
        }
    }

    #[must_use]
    pub fn invalid_params(
        message: impl Into<String>,
        field: Option<&str>,
        reason: Option<&str>,
    ) -> Self {
        let mut data = serde_json::Map::new();
        if let Some(f) = field {
            data.insert("field".to_string(), serde_json::json!(f));
        }
        if let Some(r) = reason {
            data.insert("reason".to_string(), serde_json::json!(r));
        }
        let data = if data.is_empty() {
            None
        } else {
            Some(serde_json::Value::Object(data))
        };
        Self {
            code: ErrorCode::InvalidParams as i32,
            message: message.into(),
            data,
        }
    }

    #[must_use]
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::InternalError as i32,
            message: message.into(),
            data: None,
        }
    }

    #[must_use]
    pub fn index_rebuilding() -> Self {
        Self {
            code: ErrorCode::IndexRebuilding as i32,
            message: "index is rebuilding, retry after delay".to_string(),
            data: None,
        }
    }

    #[must_use]
    pub fn pattern_too_broad(message: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::PatternTooBroad as i32,
            message: message.into(),
            data: None,
        }
    }

    #[must_use]
    pub fn permission_denied(path: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::PermissionDenied as i32,
            message: format!("permission denied: {}", path.into()),
            data: None,
        }
    }

    #[must_use]
    pub fn feature_not_supported(
        message: impl Into<String>,
        suggestion: Option<&str>,
        original_flag: Option<&str>,
    ) -> Self {
        let mut data = serde_json::Map::new();
        if let Some(s) = suggestion {
            data.insert("suggestion".to_string(), serde_json::json!(s));
        }
        if let Some(f) = original_flag {
            data.insert("originalFlag".to_string(), serde_json::json!(f));
        }
        let data = if data.is_empty() {
            None
        } else {
            Some(serde_json::Value::Object(data))
        };
        Self {
            code: ErrorCode::FeatureNotSupported as i32,
            message: message.into(),
            data,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ErrorCode {
    ParseError = -32700,
    InvalidRequest = -32600,
    MethodNotFound = -32601,
    InvalidParams = -32602,
    InternalError = -32603,
    IndexRebuilding = -1,
    PatternTooBroad = -2,
    PermissionDenied = -3,
    FeatureNotSupported = -4,
}

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid message: {0}")]
    InvalidMessage(String),

    #[error("message too large: {size} bytes (max {max})")]
    MessageTooLarge { size: usize, max: usize },

    #[error("transport error: {0}")]
    Transport(String),
}

impl JsonRpcMessage {
    pub fn from_json(data: &[u8]) -> Result<Self, ProtocolError> {
        let value: serde_json::Value = serde_json::from_slice(data)?;

        if value.get("method").is_some() && value.get("id").is_some() {
            let request: JsonRpcRequest = serde_json::from_value(value)?;
            Ok(Self::Request(request))
        } else if value.get("method").is_some() {
            let notification: JsonRpcNotification = serde_json::from_value(value)?;
            Ok(Self::Notification(notification))
        } else if value.get("id").is_some() {
            let response: JsonRpcResponse = serde_json::from_value(value)?;
            Ok(Self::Response(response))
        } else {
            Err(ProtocolError::InvalidMessage(
                "message must have either 'method' (request/notification) or 'id' (response)"
                    .to_string(),
            ))
        }
    }

    pub fn to_json(&self) -> Result<Vec<u8>, ProtocolError> {
        match self {
            Self::Request(req) => Ok(serde_json::to_vec(req)?),
            Self::Response(resp) => Ok(serde_json::to_vec(resp)?),
            Self::Notification(notif) => Ok(serde_json::to_vec(notif)?),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_roundtrip() {
        let request = JsonRpcRequest::new(
            1,
            "grep",
            serde_json::json!({
                "pattern": "TODO",
                "caseMode": "smart"
            }),
        );

        let json = serde_json::to_vec(&request).unwrap();
        let decoded: JsonRpcRequest = serde_json::from_slice(&json).unwrap();

        assert_eq!(decoded.jsonrpc, "2.0");
        assert_eq!(decoded.id, 1);
        assert_eq!(decoded.method, "grep");
        assert_eq!(decoded.params["pattern"], "TODO");
        assert_eq!(decoded.params["caseMode"], "smart");
    }

    #[test]
    fn response_success_roundtrip() {
        let response = JsonRpcResponse::success(
            1,
            serde_json::json!({
                "totalMatched": 183,
                "elapsedMs": 52
            }),
        );

        let json = serde_json::to_vec(&response).unwrap();
        let decoded: JsonRpcResponse = serde_json::from_slice(&json).unwrap();

        assert_eq!(decoded.id, Some(1));
        assert!(decoded.result.is_some());
        assert!(decoded.error.is_none());
        assert_eq!(decoded.result.unwrap()["totalMatched"], 183);
    }

    #[test]
    fn response_error_roundtrip() {
        let response = JsonRpcResponse::error(
            1,
            JsonRpcError::invalid_params("bad pattern", Some("pattern"), Some("unclosed")),
        );

        let json = serde_json::to_vec(&response).unwrap();
        let decoded: JsonRpcResponse = serde_json::from_slice(&json).unwrap();

        assert_eq!(decoded.id, Some(1));
        assert!(decoded.result.is_none());
        assert!(decoded.error.is_some());
        let err = decoded.error.unwrap();
        assert_eq!(err.code, -32602);
        assert_eq!(err.message, "bad pattern");
    }

    #[test]
    fn response_parse_error_has_null_id() {
        let response = JsonRpcResponse::parse_error(JsonRpcError::parse_error("invalid JSON"));

        let json = serde_json::to_vec(&response).unwrap();
        let decoded: JsonRpcResponse = serde_json::from_slice(&json).unwrap();

        assert_eq!(decoded.id, None);
        assert!(decoded.result.is_none());
        assert!(decoded.error.is_some());
        let err = decoded.error.unwrap();
        assert_eq!(err.code, -32700);
    }

    #[test]
    fn notification_roundtrip() {
        let notification = JsonRpcNotification::result(serde_json::json!([
            {
                "path": "src/main.rs",
                "lineNumber": 42,
                "column": 8,
                "lineContent": "    // TODO: implement",
                "matchRanges": [[8, 12]]
            }
        ]));

        let json = serde_json::to_vec(&notification).unwrap();
        let decoded: JsonRpcNotification = serde_json::from_slice(&json).unwrap();

        assert_eq!(decoded.jsonrpc, "2.0");
        assert_eq!(decoded.method, "result");
        assert!(decoded.params["items"].is_array());
    }

    #[test]
    fn message_from_json_request() {
        let json = br#"{"jsonrpc":"2.0","id":1,"method":"ping","params":{}}"#;
        let msg = JsonRpcMessage::from_json(json).unwrap();
        assert!(matches!(msg, JsonRpcMessage::Request(_)));
    }

    #[test]
    fn message_from_json_notification() {
        let json = br#"{"jsonrpc":"2.0","method":"result","params":{"items":[]}}"#;
        let msg = JsonRpcMessage::from_json(json).unwrap();
        assert!(matches!(msg, JsonRpcMessage::Notification(_)));
    }

    #[test]
    fn message_from_json_response() {
        let json = br#"{"jsonrpc":"2.0","id":1,"result":{"status":"ok"}}"#;
        let msg = JsonRpcMessage::from_json(json).unwrap();
        assert!(matches!(msg, JsonRpcMessage::Response(_)));
    }

    #[test]
    fn message_from_json_invalid() {
        let json = br#"{"jsonrpc":"2.0"}"#;
        let result = JsonRpcMessage::from_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn message_from_json_bad_json() {
        let json = b"not json at all";
        let result = JsonRpcMessage::from_json(json);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProtocolError::JsonParse(_)));
    }

    #[test]
    fn error_codes_match_spec() {
        assert_eq!(ErrorCode::ParseError as i32, -32700);
        assert_eq!(ErrorCode::InvalidRequest as i32, -32600);
        assert_eq!(ErrorCode::MethodNotFound as i32, -32601);
        assert_eq!(ErrorCode::InvalidParams as i32, -32602);
        assert_eq!(ErrorCode::InternalError as i32, -32603);
        assert_eq!(ErrorCode::IndexRebuilding as i32, -1);
        assert_eq!(ErrorCode::PatternTooBroad as i32, -2);
        assert_eq!(ErrorCode::PermissionDenied as i32, -3);
        assert_eq!(ErrorCode::FeatureNotSupported as i32, -4);
    }

    #[test]
    fn error_constructors() {
        let err = JsonRpcError::parse_error("bad json");
        assert_eq!(err.code, -32700);

        let err = JsonRpcError::method_not_found("unknown");
        assert_eq!(err.code, -32601);
        assert!(err.message.contains("unknown"));

        let err = JsonRpcError::index_rebuilding();
        assert_eq!(err.code, -1);

        let err =
            JsonRpcError::feature_not_supported("no PCRE2", Some("rg --pcre2"), Some("--pcre2"));
        assert_eq!(err.code, -4);
        let data = err.data.unwrap();
        assert_eq!(data["suggestion"], "rg --pcre2");
        assert_eq!(data["originalFlag"], "--pcre2");

        let err =
            JsonRpcError::invalid_params("bad regex", Some("pattern"), Some("unclosed group"));
        assert_eq!(err.code, -32602);
        let data = err.data.unwrap();
        assert_eq!(data["field"], "pattern");
        assert_eq!(data["reason"], "unclosed group");
    }

    #[test]
    fn response_omits_none_fields() {
        let response = JsonRpcResponse::success(1, serde_json::json!({"ok": true}));
        let json = serde_json::to_string(&response).unwrap();
        assert!(!json.contains("error"));

        let response = JsonRpcResponse::error(1, JsonRpcError::internal_error("fail"));
        let json = serde_json::to_string(&response).unwrap();
        assert!(!json.contains("result"));
    }
}
