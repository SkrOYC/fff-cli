use serde::{Deserialize, Serialize};

use ff_common::{CaseMode, ExecMode, FileType, PatternMode};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrepParams {
    pub pattern: String,
    #[serde(default)]
    pub case_mode: CaseMode,
    #[serde(default)]
    pub context_lines: Option<ContextLines>,
    #[serde(default)]
    pub filters: Option<FileFilters>,
    #[serde(default)]
    pub max_matches: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextLines {
    #[serde(default)]
    pub before: u32,
    #[serde(default)]
    pub after: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileFilters {
    #[serde(default)]
    pub extensions: Vec<String>,
    #[serde(default)]
    pub globs: Vec<String>,
    #[serde(default)]
    pub file_types: Vec<FileTypeFilter>,
    #[serde(default)]
    pub excludes: Vec<String>,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub no_ignore: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FileTypeFilter {
    File,
    Directory,
    Symlink,
    Socket,
    BlockDevice,
    CharacterDevice,
    Fifo,
    Executable,
    Empty,
}

impl From<FileTypeFilter> for FileType {
    fn from(ft: FileTypeFilter) -> Self {
        match ft {
            FileTypeFilter::File => FileType::RegularFile,
            FileTypeFilter::Directory => FileType::Directory,
            FileTypeFilter::Symlink => FileType::Symlink,
            FileTypeFilter::Socket => FileType::Socket,
            FileTypeFilter::BlockDevice => FileType::BlockDevice,
            FileTypeFilter::CharacterDevice => FileType::CharacterDevice,
            FileTypeFilter::Fifo => FileType::Fifo,
            FileTypeFilter::Executable | FileTypeFilter::Empty => FileType::RegularFile,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchParams {
    pub pattern: String,
    #[serde(default)]
    pub pattern_mode: PatternMode,
    #[serde(default)]
    pub filters: Option<FileFilters>,
    #[serde(default)]
    pub max_depth: Option<u32>,
    #[serde(default)]
    pub min_depth: Option<u32>,
    #[serde(default)]
    pub max_matches: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FindParams {
    pub expression: String,
    #[serde(default)]
    pub actions: Vec<ActionSpec>,
    #[serde(default)]
    pub max_depth: Option<u32>,
    #[serde(default)]
    pub min_depth: Option<u32>,
    #[serde(default)]
    pub follow_symlinks: bool,
    #[serde(default)]
    pub mount_boundary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionSpec {
    #[serde(rename = "type")]
    pub action_type: ActionType,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub mode: Option<ExecMode>,
    #[serde(default)]
    pub format: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionType {
    Print,
    Print0,
    Printf,
    Ls,
    Exec,
    Execdir,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingParams {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownParams {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrepResultItem {
    pub path: String,
    pub line_number: u32,
    pub column: u32,
    pub line_content: String,
    pub match_ranges: Vec<(u32, u32)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultItem {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtime: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FindResultItem {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtime: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inode: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_count: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuerySummary {
    pub total_matched: u64,
    pub elapsed_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FindQuerySummary {
    pub total_matched: u64,
    pub elapsed_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_results: Option<Vec<ActionResult>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionResult {
    #[serde(rename = "type")]
    pub action_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PingResult {
    pub status: String,
    pub version: String,
    pub root_path: String,
    pub indexed_files: u64,
    pub uptime_seconds: u64,
    pub index_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownResult {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamNotificationParams {
    pub items: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grep_params_roundtrip() {
        let params = GrepParams {
            pattern: "TODO".to_string(),
            case_mode: CaseMode::Smart,
            context_lines: Some(ContextLines {
                before: 2,
                after: 2,
            }),
            filters: Some(FileFilters {
                extensions: vec!["rs".to_string(), "ts".to_string()],
                globs: vec!["src/**".to_string()],
                file_types: vec![FileTypeFilter::File],
                excludes: vec![],
                hidden: false,
                no_ignore: false,
            }),
            max_matches: Some(10000),
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("caseMode"));
        assert!(json.contains("contextLines"));
        assert!(json.contains("maxMatches"));
        assert!(json.contains("fileTypes"));
        assert!(json.contains("noIgnore"));

        let decoded: GrepParams = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.pattern, "TODO");
        assert_eq!(decoded.case_mode, CaseMode::Smart);
        assert_eq!(decoded.context_lines.unwrap().before, 2);
        assert_eq!(decoded.filters.unwrap().extensions.len(), 2);
    }

    #[test]
    fn search_params_roundtrip() {
        let params = SearchParams {
            pattern: r"\.rs$".to_string(),
            pattern_mode: PatternMode::Regex,
            filters: None,
            max_depth: Some(10),
            min_depth: Some(0),
            max_matches: Some(10000),
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("patternMode"));
        assert!(json.contains("maxDepth"));
        assert!(json.contains("minDepth"));
        assert!(json.contains("maxMatches"));

        let decoded: SearchParams = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.pattern, r"\.rs$");
        assert_eq!(decoded.pattern_mode, PatternMode::Regex);
        assert_eq!(decoded.max_depth, Some(10));
    }

    #[test]
    fn find_params_roundtrip() {
        let params = FindParams {
            expression: "-name *.rs -type f -size +1k".to_string(),
            actions: vec![ActionSpec {
                action_type: ActionType::Exec,
                command: Some("wc -l".to_string()),
                mode: Some(ExecMode::Batch),
                format: None,
            }],
            max_depth: Some(10),
            min_depth: Some(0),
            follow_symlinks: false,
            mount_boundary: false,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("maxDepth"));
        assert!(json.contains("followSymlinks"));
        assert!(json.contains("mountBoundary"));

        let decoded: FindParams = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.expression, "-name *.rs -type f -size +1k");
        assert_eq!(decoded.actions.len(), 1);
        assert_eq!(decoded.actions[0].action_type, ActionType::Exec);
    }

    #[test]
    fn grep_result_item_camel_case() {
        let item = GrepResultItem {
            path: "src/main.rs".to_string(),
            line_number: 42,
            column: 8,
            line_content: "    // TODO: implement".to_string(),
            match_ranges: vec![(8, 12)],
            git_status: Some("modified".to_string()),
            file_type: Some("file".to_string()),
        };

        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("lineNumber"));
        assert!(json.contains("lineContent"));
        assert!(json.contains("matchRanges"));
        assert!(json.contains("gitStatus"));
        assert!(json.contains("fileType"));

        let decoded: GrepResultItem = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.line_number, 42);
        assert_eq!(decoded.match_ranges, vec![(8, 12)]);
    }

    #[test]
    fn query_summary_camel_case() {
        let summary = QuerySummary {
            total_matched: 183,
            elapsed_ms: 52,
        };

        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("totalMatched"));
        assert!(json.contains("elapsedMs"));

        let decoded: QuerySummary = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.total_matched, 183);
        assert_eq!(decoded.elapsed_ms, 52);
    }

    #[test]
    fn ping_result_camel_case() {
        let result = PingResult {
            status: "ok".to_string(),
            version: "0.1.0".to_string(),
            root_path: "/home/user/project".to_string(),
            indexed_files: 54321,
            uptime_seconds: 3600,
            index_status: "ready".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("rootPath"));
        assert!(json.contains("indexedFiles"));
        assert!(json.contains("uptimeSeconds"));
        assert!(json.contains("indexStatus"));

        let decoded: PingResult = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.status, "ok");
        assert_eq!(decoded.indexed_files, 54321);
    }

    #[test]
    fn file_type_filter_kebab_case() {
        let json = serde_json::to_string(&FileTypeFilter::BlockDevice).unwrap();
        assert_eq!(json, "\"block-device\"");

        let json = serde_json::to_string(&FileTypeFilter::CharacterDevice).unwrap();
        assert_eq!(json, "\"character-device\"");

        let decoded: FileTypeFilter = serde_json::from_str("\"block-device\"").unwrap();
        assert_eq!(decoded, FileTypeFilter::BlockDevice);
    }

    #[test]
    fn file_type_filter_conversion() {
        assert_eq!(FileType::from(FileTypeFilter::File), FileType::RegularFile);
        assert_eq!(
            FileType::from(FileTypeFilter::Directory),
            FileType::Directory
        );
        assert_eq!(FileType::from(FileTypeFilter::Symlink), FileType::Symlink);
    }

    #[test]
    fn grep_params_defaults() {
        let json = r#"{"pattern": "test"}"#;
        let params: GrepParams = serde_json::from_str(json).unwrap();

        assert_eq!(params.pattern, "test");
        assert_eq!(params.case_mode, CaseMode::Smart);
        assert!(params.context_lines.is_none());
        assert!(params.filters.is_none());
        assert!(params.max_matches.is_none());
    }

    #[test]
    fn case_mode_serializes_lowercase() {
        let json = serde_json::to_string(&CaseMode::Smart).unwrap();
        assert_eq!(json, "\"smart\"");

        let json = serde_json::to_string(&CaseMode::Sensitive).unwrap();
        assert_eq!(json, "\"sensitive\"");

        let json = serde_json::to_string(&CaseMode::Insensitive).unwrap();
        assert_eq!(json, "\"insensitive\"");
    }

    #[test]
    fn pattern_mode_serializes_kebab_case() {
        let json = serde_json::to_string(&PatternMode::Regex).unwrap();
        assert_eq!(json, "\"regex\"");

        let json = serde_json::to_string(&PatternMode::FixedString).unwrap();
        assert_eq!(json, "\"fixed-string\"");
    }

    #[test]
    fn exec_mode_serializes_kebab_case() {
        let json = serde_json::to_string(&ExecMode::PerMatch).unwrap();
        assert_eq!(json, "\"per-match\"");

        let json = serde_json::to_string(&ExecMode::PerMatchDir).unwrap();
        assert_eq!(json, "\"per-match-dir\"");
    }

    #[test]
    fn stream_notification_params() {
        let params = StreamNotificationParams {
            items: serde_json::json!([
                {"path": "file.rs", "lineNumber": 1}
            ]),
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("items"));
    }

    #[test]
    fn action_result_camel_case() {
        let result = ActionResult {
            action_type: "exec".to_string(),
            exit_code: Some(0),
            stdout: Some("output".to_string()),
            stderr: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("exitCode"));
    }
}
