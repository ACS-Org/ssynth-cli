use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Auth ──

#[derive(Debug, Serialize)]
pub struct DevLoginRequest {
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub user: UserInfo,
}

#[derive(Debug, Deserialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub github_login: String,
    pub tenant_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct TokenExchangeResponse {
    pub access_token: String,
}

// ── Tenant ──

#[derive(Debug, Serialize, Deserialize)]
pub struct Tenant {
    pub id: Uuid,
    pub slug: String,
    pub display_name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── Project ──

#[derive(Debug, Serialize)]
pub struct CreateProjectRequest {
    pub slug: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_target_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub slug: String,
    pub display_name: String,
    pub default_target_id: Option<Uuid>,
    pub retention_days: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── Target ──

#[derive(Debug, Serialize, Deserialize)]
pub struct Target {
    pub id: Uuid,
    pub family: String,
    pub device: String,
    pub package: Option<String>,
    pub board: Option<String>,
    pub toolchain_lane: String,
    pub created_at: DateTime<Utc>,
}

// ── Job ──

#[derive(Debug, Serialize)]
pub struct CreateJobRequest {
    pub target_id: Uuid,
    pub source_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_upload_key: Option<String>,
    pub top_module: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraint_files: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_args: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_seeds: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_pick: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compute_parallelism: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compute_priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_steps: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub project_id: Uuid,
    pub target_id: Uuid,
    pub submitted_by: Uuid,
    pub source_type: String,
    pub top_module: String,
    pub search_seeds: i32,
    pub search_pick: String,
    pub compute_parallelism: i32,
    pub compute_priority: String,
    pub requested_steps: Vec<String>,
    pub status: String,
    pub idempotency_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobDetailResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub project_id: Uuid,
    pub target_id: Uuid,
    pub submitted_by: Uuid,
    pub source_type: String,
    pub source_upload_key: Option<String>,
    pub top_module: String,
    pub search_seeds: i32,
    pub search_pick: String,
    pub compute_parallelism: i32,
    pub compute_priority: String,
    pub requested_steps: Vec<String>,
    pub status: String,
    pub idempotency_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub runs: Vec<RunWithSteps>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunWithSteps {
    pub id: Uuid,
    pub job_id: Uuid,
    pub seed: i32,
    pub attempt: i32,
    pub status: String,
    pub timing_mhz: Option<f64>,
    pub area_luts: Option<i64>,
    pub area_ffs: Option<i64>,
    pub is_winner: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub steps: Vec<RunStep>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunStep {
    pub id: Uuid,
    pub run_id: Uuid,
    pub step_name: String,
    pub ordinal: i32,
    pub status: String,
    pub exit_code: Option<i32>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_secs: Option<f64>,
}

// ── Upload ──

#[derive(Debug, Deserialize)]
pub struct UploadResponse {
    pub source_key: String,
}

// ── Artifacts ──

#[derive(Debug, Serialize, Deserialize)]
pub struct Artifact {
    pub id: Uuid,
    pub run_id: Uuid,
    pub job_id: Uuid,
    #[serde(alias = "artifact_type")]
    pub kind: String,
    pub filename: String,
    pub size_bytes: i64,
    pub sha256: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ── Logs ──

#[derive(Debug, Serialize, Deserialize)]
pub struct LogLine {
    pub line_num: i64,
    pub stream: String,
    pub content: String,
}

// ── WebSocket messages ──

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    #[serde(rename = "log")]
    Log {
        line_num: i64,
        stream: String,
        content: String,
    },
    #[serde(rename = "status")]
    Status { status: String },
}

// ── API Error ──

#[derive(Debug, Deserialize)]
pub struct ApiErrorResponse {
    pub error: String,
    pub message: String,
}
