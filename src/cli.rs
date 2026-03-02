// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "ssynth",
    about = "CLI for SuperSynth FPGA synthesis platform",
    version
)]
pub struct Cli {
    /// Output as JSON instead of human-readable tables
    #[arg(long, global = true)]
    pub json: bool,

    /// Override the API URL
    #[arg(long, global = true, env = "SSYNTH_API_URL")]
    pub api_url: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Authenticate with the `SuperSynth` API
    Login(LoginArgs),

    /// Manage synthesis jobs
    #[command(subcommand)]
    Job(Box<JobCommand>),

    /// Manage build artifacts
    #[command(subcommand)]
    Artifact(ArtifactCommand),

    /// Manage projects
    #[command(subcommand)]
    Project(ProjectCommand),

    /// Manage API keys
    #[command(subcommand, name = "api-key")]
    ApiKey(ApiKeyCommand),

    /// Show credit balance and usage
    Usage,

    /// List available FPGA targets
    Targets,

    /// Show current configuration
    Config(ConfigArgs),
}

// ── Login ──

#[derive(Parser)]
pub struct LoginArgs {
    /// API key (`sk_live`_...). If omitted, you'll be prompted.
    #[arg(long)]
    pub api_key: Option<String>,

    /// Use dev-login instead of API key auth
    #[arg(long)]
    pub dev: bool,

    /// Username for dev-login
    #[arg(long)]
    pub username: Option<String>,
}

// ── Job ──

#[derive(Subcommand)]
pub enum JobCommand {
    /// Submit a new synthesis job
    Submit(JobSubmitArgs),
    /// Show job status and run details
    Status(JobStatusArgs),
    /// List jobs
    List(JobListArgs),
    /// View job logs
    Logs(JobLogsArgs),
    /// Cancel a running job
    Cancel(JobCancelArgs),
    /// Retry failed (or all) seeds of a completed/failed job
    Retry(JobRetryArgs),
    /// Clone a job with optional parameter overrides
    Clone(JobCloneArgs),
}

#[derive(Parser)]
pub struct JobSubmitArgs {
    /// Path to source directory or prepared bundle (.tar.gz / .zip)
    pub path: String,

    /// Project ID
    #[arg(long)]
    pub project: Option<String>,

    /// Target ID (auto-detected from hwbuild.yml if not set)
    #[arg(long)]
    pub target: Option<String>,

    /// Top module name
    #[arg(long)]
    pub top: Option<String>,

    /// Constraint files
    #[arg(long, num_args = 1..)]
    pub constraints: Option<Vec<String>>,

    /// Number of seeds to search
    #[arg(long)]
    pub seeds: Option<i32>,

    /// Seed selection strategy: `best_timing` or `best_area`
    #[arg(long)]
    pub pick: Option<String>,

    /// Compute priority: interactive, standard, or batch
    #[arg(long)]
    pub priority: Option<String>,

    /// Parallelism level
    #[arg(long)]
    pub parallelism: Option<i32>,

    /// Pipeline steps (e.g. synth,pnr,bitstream)
    #[arg(long, value_delimiter = ',')]
    pub steps: Option<Vec<String>>,

    /// Max runtime (e.g., "2h", "30m", "1h30m", "86400s")
    #[arg(long)]
    pub max_runtime: Option<String>,

    /// Max memory (e.g., "16GB", "4096MB")
    #[arg(long)]
    pub max_memory: Option<String>,

    /// Wait for job to complete, streaming logs
    #[arg(long)]
    pub wait: bool,

    /// Idempotency key for deduplication
    #[arg(long)]
    pub idempotency_key: Option<String>,
}

#[derive(Parser)]
pub struct JobStatusArgs {
    /// Job ID
    pub job_id: String,

    /// Watch mode: refresh every 5 seconds
    #[arg(long)]
    pub watch: bool,
}

#[derive(Parser)]
pub struct JobListArgs {
    /// Filter by status
    #[arg(long)]
    pub status: Option<String>,

    /// Filter by project ID
    #[arg(long)]
    pub project: Option<String>,

    /// Maximum number of results
    #[arg(long)]
    pub limit: Option<i32>,
}

#[derive(Parser)]
pub struct JobLogsArgs {
    /// Job ID
    pub job_id: String,

    /// Follow log output via WebSocket
    #[arg(long)]
    pub follow: bool,

    /// Starting line offset
    #[arg(long)]
    pub offset: Option<i64>,

    /// Maximum number of lines
    #[arg(long)]
    pub limit: Option<i64>,
}

#[derive(Parser)]
pub struct JobCancelArgs {
    /// Job ID
    pub job_id: String,
}

#[derive(Parser)]
pub struct JobRetryArgs {
    /// Job ID
    pub job_id: String,

    /// Retry scope: "failed" (default) or "all"
    #[arg(long, default_value = "failed")]
    pub scope: String,
}

#[derive(Parser)]
pub struct JobCloneArgs {
    /// Job ID to clone from
    pub job_id: String,

    /// Number of seeds (inherits from parent if not set)
    #[arg(long)]
    pub seeds: Option<i32>,

    /// Parallelism level
    #[arg(long)]
    pub parallelism: Option<i32>,

    /// Compute priority: interactive, standard, or batch
    #[arg(long)]
    pub priority: Option<String>,

    /// Seed selection strategy: `best_timing` or `best_area`
    #[arg(long)]
    pub pick: Option<String>,

    /// Target ID
    #[arg(long)]
    pub target: Option<String>,
}

// ── Artifact ──

#[derive(Subcommand)]
pub enum ArtifactCommand {
    /// List artifacts for a job
    List(ArtifactListArgs),
    /// Download artifacts
    Download(ArtifactDownloadArgs),
}

#[derive(Parser)]
pub struct ArtifactListArgs {
    /// Job ID
    pub job_id: String,
}

#[derive(Parser)]
pub struct ArtifactDownloadArgs {
    /// Job ID
    pub job_id: String,

    /// Output directory (default: current directory)
    #[arg(long)]
    pub output_dir: Option<String>,

    /// Download a specific artifact by ID (default: download all)
    #[arg(long)]
    pub artifact_id: Option<String>,
}

// ── Project ──

#[derive(Subcommand)]
pub enum ProjectCommand {
    /// List projects
    List(ProjectListArgs),
    /// Create a new project
    Create(ProjectCreateArgs),
    /// Get project details
    Get(ProjectGetArgs),
    /// Update a project
    Update(ProjectUpdateArgs),
    /// Delete a project
    Delete(ProjectDeleteArgs),
}

#[derive(Parser)]
pub struct ProjectListArgs {}

#[derive(Parser)]
pub struct ProjectCreateArgs {
    /// Project slug (URL-safe identifier)
    #[arg(long)]
    pub slug: String,

    /// Display name
    #[arg(long)]
    pub name: String,

    /// Default target ID
    #[arg(long)]
    pub target: Option<String>,
}

#[derive(Parser)]
pub struct ProjectGetArgs {
    /// Project ID
    pub id: String,
}

#[derive(Parser)]
pub struct ProjectUpdateArgs {
    /// Project ID
    pub id: String,

    /// New display name
    #[arg(long)]
    pub name: Option<String>,

    /// Retention days
    #[arg(long)]
    pub retention_days: Option<i32>,
}

#[derive(Parser)]
pub struct ProjectDeleteArgs {
    /// Project ID
    pub id: String,
}

// ── API Key ──

#[derive(Subcommand)]
pub enum ApiKeyCommand {
    /// Create a new API key
    Create(ApiKeyCreateArgs),
    /// List API keys
    List,
    /// Revoke an API key
    Revoke(ApiKeyRevokeArgs),
}

#[derive(Parser)]
pub struct ApiKeyCreateArgs {
    /// Key name
    #[arg(long)]
    pub name: String,

    /// Expiration date (ISO 8601, e.g. 2025-12-31)
    #[arg(long)]
    pub expires_at: Option<String>,
}

#[derive(Parser)]
pub struct ApiKeyRevokeArgs {
    /// API key ID
    pub id: String,
}

// ── Config ──

#[derive(Parser)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub action: ConfigAction,
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Show current configuration
    Show,
}
