use std::time::Instant;

use ff_ipc::{
    FindParams, GrepParams, JsonRpcError, JsonRpcNotification, JsonRpcResponse, QuerySummary,
    SearchParams,
};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryType {
    Grep,
    Search,
    Find,
    Ping,
    Shutdown,
}

impl QueryType {
    #[must_use]
    pub fn from_method(method: &str) -> Option<Self> {
        match method {
            "grep" => Some(Self::Grep),
            "search" => Some(Self::Search),
            "find" => Some(Self::Find),
            "ping" => Some(Self::Ping),
            "shutdown" => Some(Self::Shutdown),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct QueryResult {
    pub notifications: Vec<JsonRpcNotification>,
    pub response: JsonRpcResponse,
    pub elapsed_ms: u64,
}

pub trait QueryDispatcher: Send + Sync {
    fn dispatch_grep(
        &self,
        id: u64,
        params: GrepParams,
    ) -> impl std::future::Future<Output = QueryResult> + Send;

    fn dispatch_search(
        &self,
        id: u64,
        params: SearchParams,
    ) -> impl std::future::Future<Output = QueryResult> + Send;

    fn dispatch_find(
        &self,
        id: u64,
        params: FindParams,
    ) -> impl std::future::Future<Output = QueryResult> + Send;

    fn dispatch_ping(&self, id: u64) -> QueryResult;

    fn dispatch_shutdown(&self, id: u64) -> QueryResult;
}

pub struct StubDispatcher;

impl QueryDispatcher for StubDispatcher {
    async fn dispatch_grep(&self, id: u64, _params: GrepParams) -> QueryResult {
        let start = Instant::now();
        let summary = QuerySummary {
            total_matched: 0,
            elapsed_ms: start.elapsed().as_millis() as u64,
        };
        QueryResult {
            notifications: vec![],
            response: JsonRpcResponse::success(id, serde_json::to_value(summary).unwrap()),
            elapsed_ms: start.elapsed().as_millis() as u64,
        }
    }

    async fn dispatch_search(&self, id: u64, _params: SearchParams) -> QueryResult {
        let start = Instant::now();
        let summary = QuerySummary {
            total_matched: 0,
            elapsed_ms: start.elapsed().as_millis() as u64,
        };
        QueryResult {
            notifications: vec![],
            response: JsonRpcResponse::success(id, serde_json::to_value(summary).unwrap()),
            elapsed_ms: start.elapsed().as_millis() as u64,
        }
    }

    async fn dispatch_find(&self, id: u64, _params: FindParams) -> QueryResult {
        let start = Instant::now();
        let summary = QuerySummary {
            total_matched: 0,
            elapsed_ms: start.elapsed().as_millis() as u64,
        };
        QueryResult {
            notifications: vec![],
            response: JsonRpcResponse::success(id, serde_json::to_value(summary).unwrap()),
            elapsed_ms: start.elapsed().as_millis() as u64,
        }
    }

    fn dispatch_ping(&self, id: u64) -> QueryResult {
        let result = serde_json::json!({
            "status": "ok",
            "version": env!("CARGO_PKG_VERSION"),
            "rootPath": "",
            "indexedFiles": 0,
            "uptimeSeconds": 0,
            "indexStatus": "ready"
        });
        QueryResult {
            notifications: vec![],
            response: JsonRpcResponse::success(id, result),
            elapsed_ms: 0,
        }
    }

    fn dispatch_shutdown(&self, id: u64) -> QueryResult {
        let result = serde_json::json!({
            "status": "shutting_down"
        });
        QueryResult {
            notifications: vec![],
            response: JsonRpcResponse::success(id, result),
            elapsed_ms: 0,
        }
    }
}

pub fn parse_query_params(method: &str, params: Value) -> Result<QueryParams, JsonRpcError> {
    match method {
        "grep" => {
            let p: GrepParams = serde_json::from_value(params)
                .map_err(|e| JsonRpcError::invalid_params(e.to_string(), Some("params"), None))?;
            Ok(QueryParams::Grep(p))
        }
        "search" => {
            let p: SearchParams = serde_json::from_value(params)
                .map_err(|e| JsonRpcError::invalid_params(e.to_string(), Some("params"), None))?;
            Ok(QueryParams::Search(p))
        }
        "find" => {
            let p: FindParams = serde_json::from_value(params)
                .map_err(|e| JsonRpcError::invalid_params(e.to_string(), Some("params"), None))?;
            Ok(QueryParams::Find(p))
        }
        "ping" | "shutdown" => Ok(QueryParams::Empty),
        _ => Err(JsonRpcError::method_not_found(method)),
    }
}

#[derive(Debug)]
pub enum QueryParams {
    Grep(GrepParams),
    Search(SearchParams),
    Find(FindParams),
    Empty,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_type_from_method() {
        assert_eq!(QueryType::from_method("grep"), Some(QueryType::Grep));
        assert_eq!(QueryType::from_method("search"), Some(QueryType::Search));
        assert_eq!(QueryType::from_method("find"), Some(QueryType::Find));
        assert_eq!(QueryType::from_method("ping"), Some(QueryType::Ping));
        assert_eq!(
            QueryType::from_method("shutdown"),
            Some(QueryType::Shutdown)
        );
        assert_eq!(QueryType::from_method("unknown"), None);
    }

    #[tokio::test]
    async fn stub_dispatcher_ping() {
        let dispatcher = StubDispatcher;
        let result = dispatcher.dispatch_ping(1);
        assert_eq!(result.response.id, Some(1));
        assert!(result.response.result.is_some());
        let result_val = result.response.result.unwrap();
        assert_eq!(result_val["status"], "ok");
    }

    #[tokio::test]
    async fn stub_dispatcher_shutdown() {
        let dispatcher = StubDispatcher;
        let result = dispatcher.dispatch_shutdown(2);
        assert_eq!(result.response.id, Some(2));
        let result_val = result.response.result.unwrap();
        assert_eq!(result_val["status"], "shutting_down");
    }

    #[tokio::test]
    async fn stub_dispatcher_grep() {
        let dispatcher = StubDispatcher;
        let params = GrepParams {
            pattern: "test".to_string(),
            case_mode: ff_common::CaseMode::Smart,
            context_lines: None,
            filters: None,
            max_matches: None,
        };
        let result = dispatcher.dispatch_grep(3, params).await;
        assert_eq!(result.response.id, Some(3));
        let result_val = result.response.result.unwrap();
        assert_eq!(result_val["totalMatched"], 0);
    }

    #[test]
    fn parse_grep_params() {
        let json = serde_json::json!({
            "pattern": "TODO",
            "caseMode": "smart"
        });
        let result = parse_query_params("grep", json);
        assert!(result.is_ok());
        match result.unwrap() {
            QueryParams::Grep(p) => assert_eq!(p.pattern, "TODO"),
            _ => panic!("expected grep params"),
        }
    }

    #[test]
    fn parse_search_params() {
        let json = serde_json::json!({
            "pattern": "*.rs",
            "patternMode": "glob"
        });
        let result = parse_query_params("search", json);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_unknown_method() {
        let json = serde_json::json!({});
        let result = parse_query_params("unknown", json);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, -32601);
    }

    #[test]
    fn parse_invalid_params() {
        let json = serde_json::json!({"wrong": "fields"});
        let result = parse_query_params("grep", json);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, -32602);
    }
}
