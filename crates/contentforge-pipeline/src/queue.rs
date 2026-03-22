use anyhow::Result;
use chrono::{DateTime, Utc};
use contentforge_db::DbPool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A job in the pipeline queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub pipeline: String,
    pub project: Option<String>,
    pub content_id: Option<Uuid>,
    pub step: String,
    pub platform: Option<String>,
    pub status: JobStatus,
    pub payload: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub attempts: u32,
    pub max_retries: u32,
    pub retry_after: Option<DateTime<Utc>>,
    pub parent_job: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Running,
    AwaitingReview,
    Approved,
    Published,
    Failed,
    Retrying,
    Cancelled,
    Done,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Running => write!(f, "running"),
            Self::AwaitingReview => write!(f, "awaiting_review"),
            Self::Approved => write!(f, "approved"),
            Self::Published => write!(f, "published"),
            Self::Failed => write!(f, "failed"),
            Self::Retrying => write!(f, "retrying"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::Done => write!(f, "done"),
        }
    }
}

/// Job queue backed by SQLite.
pub struct JobQueue {
    db: DbPool,
}

impl JobQueue {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    /// Create a new job and insert into the queue.
    #[allow(clippy::too_many_arguments)]
    pub fn enqueue(
        &self,
        pipeline: &str,
        step: &str,
        content_id: Option<Uuid>,
        project: Option<&str>,
        platform: Option<&str>,
        payload: serde_json::Value,
        parent_job: Option<Uuid>,
    ) -> Result<Job> {
        let job = Job {
            id: Uuid::new_v4(),
            pipeline: pipeline.to_string(),
            project: project.map(|s| s.to_string()),
            content_id,
            step: step.to_string(),
            platform: platform.map(|s| s.to_string()),
            status: JobStatus::Pending,
            payload,
            result: None,
            error: None,
            attempts: 0,
            max_retries: 3,
            retry_after: None,
            parent_job,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
        };

        let conn = self.db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
        conn.execute(
            "INSERT INTO jobs (id, pipeline, project, content_id, step, platform, status, payload, attempts, max_retries, parent_job, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            rusqlite::params![
                job.id.to_string(),
                job.pipeline,
                job.project,
                job.content_id.map(|id| id.to_string()),
                job.step,
                job.platform,
                job.status.to_string(),
                serde_json::to_string(&job.payload)?,
                job.attempts,
                job.max_retries,
                job.parent_job.map(|id| id.to_string()),
                job.created_at.to_rfc3339(),
                job.updated_at.to_rfc3339(),
            ],
        )?;

        tracing::info!(
            job_id = %job.id,
            pipeline = %job.pipeline,
            step = %job.step,
            "Job enqueued"
        );

        Ok(job)
    }

    /// Dequeue the next pending or retry-ready job.
    pub fn dequeue(&self) -> Result<Option<Job>> {
        let conn = self.db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;

        // Find next job: pending, or retrying with retry_after <= now
        let mut stmt = conn.prepare(
            "SELECT id FROM jobs
             WHERE status = 'pending'
                OR (status = 'retrying' AND (retry_after IS NULL OR retry_after <= datetime('now')))
             ORDER BY created_at ASC LIMIT 1",
        )?;

        let job_id: Option<String> = stmt.query_row([], |row| row.get(0)).optional()?;

        let job_id = match job_id {
            Some(id) => id,
            None => return Ok(None),
        };

        // Atomically mark as running
        conn.execute(
            "UPDATE jobs SET status = 'running', attempts = attempts + 1, updated_at = datetime('now') WHERE id = ?1",
            [&job_id],
        )?;

        // Load full job
        self.get_by_id_str(&job_id)
    }

    /// Get a job by UUID string.
    fn get_by_id_str(&self, id: &str) -> Result<Option<Job>> {
        let conn = self.db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
        let mut stmt = conn.prepare(
            "SELECT id, pipeline, project, content_id, step, platform, status, payload, result, error,
                    attempts, max_retries, retry_after, parent_job, created_at, updated_at, completed_at
             FROM jobs WHERE id = ?1",
        )?;

        stmt.query_row([id], |row| {
            Ok(Job {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                pipeline: row.get(1)?,
                project: row.get(2)?,
                content_id: row
                    .get::<_, Option<String>>(3)?
                    .and_then(|s| Uuid::parse_str(&s).ok()),
                step: row.get(4)?,
                platform: row.get(5)?,
                status: parse_job_status(&row.get::<_, String>(6)?),
                payload: row
                    .get::<_, Option<String>>(7)?
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or(serde_json::json!({})),
                result: row
                    .get::<_, Option<String>>(8)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                error: row.get(9)?,
                attempts: row.get::<_, i32>(10)? as u32,
                max_retries: row.get::<_, i32>(11)? as u32,
                retry_after: row
                    .get::<_, Option<String>>(12)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                parent_job: row
                    .get::<_, Option<String>>(13)?
                    .and_then(|s| Uuid::parse_str(&s).ok()),
                created_at: parse_dt(&row.get::<_, String>(14)?),
                updated_at: parse_dt(&row.get::<_, String>(15)?),
                completed_at: row.get::<_, Option<String>>(16)?.map(|s| parse_dt(&s)),
            })
        })
        .optional()
    }

    /// Get a job by UUID.
    pub fn get(&self, id: Uuid) -> Result<Option<Job>> {
        self.get_by_id_str(&id.to_string())
    }

    /// Mark a job as completed with a result.
    pub fn complete(&self, id: Uuid, result: serde_json::Value) -> Result<()> {
        let conn = self.db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
        conn.execute(
            "UPDATE jobs SET status = 'done', result = ?1, completed_at = datetime('now'), updated_at = datetime('now') WHERE id = ?2",
            rusqlite::params![serde_json::to_string(&result)?, id.to_string()],
        )?;
        Ok(())
    }

    /// Mark a job as published.
    pub fn mark_published(&self, id: Uuid, result: serde_json::Value) -> Result<()> {
        let conn = self.db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
        conn.execute(
            "UPDATE jobs SET status = 'published', result = ?1, completed_at = datetime('now'), updated_at = datetime('now') WHERE id = ?2",
            rusqlite::params![serde_json::to_string(&result)?, id.to_string()],
        )?;
        Ok(())
    }

    /// Mark a job as failed, with optional retry.
    pub fn fail(&self, id: Uuid, error: &str, can_retry: bool) -> Result<()> {
        let conn = self.db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;

        // Check current attempts vs max_retries
        let (attempts, max_retries): (i32, i32) = conn.query_row(
            "SELECT attempts, max_retries FROM jobs WHERE id = ?1",
            [id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        if can_retry && attempts < max_retries {
            let delay = next_retry_delay(attempts as u32);
            let retry_at = Utc::now() + delay;
            conn.execute(
                "UPDATE jobs SET status = 'retrying', error = ?1, retry_after = ?2, updated_at = datetime('now') WHERE id = ?3",
                rusqlite::params![error, retry_at.to_rfc3339(), id.to_string()],
            )?;
            tracing::warn!(
                job_id = %id,
                attempt = attempts,
                retry_at = %retry_at,
                "Job failed, scheduling retry"
            );
        } else {
            conn.execute(
                "UPDATE jobs SET status = 'failed', error = ?1, completed_at = datetime('now'), updated_at = datetime('now') WHERE id = ?2",
                rusqlite::params![error, id.to_string()],
            )?;
            tracing::error!(job_id = %id, error, "Job failed permanently");
        }
        Ok(())
    }

    /// Suspend a job for review.
    pub fn suspend_for_review(&self, id: Uuid) -> Result<()> {
        let conn = self.db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
        conn.execute(
            "UPDATE jobs SET status = 'awaiting_review', updated_at = datetime('now') WHERE id = ?1",
            [id.to_string()],
        )?;
        Ok(())
    }

    /// Approve a job that's awaiting review.
    pub fn approve(&self, id: Uuid) -> Result<()> {
        let conn = self.db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
        conn.execute(
            "UPDATE jobs SET status = 'approved', updated_at = datetime('now') WHERE id = ?1",
            [id.to_string()],
        )?;
        Ok(())
    }

    /// Reject a job that's awaiting review.
    pub fn reject(&self, id: Uuid, reason: &str) -> Result<()> {
        let conn = self.db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
        conn.execute(
            "UPDATE jobs SET status = 'cancelled', error = ?1, completed_at = datetime('now'), updated_at = datetime('now') WHERE id = ?2",
            rusqlite::params![reason, id.to_string()],
        )?;
        Ok(())
    }

    /// List jobs, optionally filtered by status or pipeline.
    pub fn list(
        &self,
        status: Option<&str>,
        pipeline: Option<&str>,
        limit: usize,
    ) -> Result<Vec<Job>> {
        let conn = self.db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;

        let (query, params) = match (status, pipeline) {
            (Some(s), Some(p)) => (
                "SELECT id, pipeline, project, content_id, step, platform, status, payload, result, error,
                        attempts, max_retries, retry_after, parent_job, created_at, updated_at, completed_at
                 FROM jobs WHERE status = ?1 AND pipeline = ?2 ORDER BY created_at DESC LIMIT ?3",
                vec![s.to_string(), p.to_string(), limit.to_string()],
            ),
            (Some(s), None) => (
                "SELECT id, pipeline, project, content_id, step, platform, status, payload, result, error,
                        attempts, max_retries, retry_after, parent_job, created_at, updated_at, completed_at
                 FROM jobs WHERE status = ?1 ORDER BY created_at DESC LIMIT ?2",
                vec![s.to_string(), limit.to_string()],
            ),
            (None, Some(p)) => (
                "SELECT id, pipeline, project, content_id, step, platform, status, payload, result, error,
                        attempts, max_retries, retry_after, parent_job, created_at, updated_at, completed_at
                 FROM jobs WHERE pipeline = ?1 ORDER BY created_at DESC LIMIT ?2",
                vec![p.to_string(), limit.to_string()],
            ),
            (None, None) => (
                "SELECT id, pipeline, project, content_id, step, platform, status, payload, result, error,
                        attempts, max_retries, retry_after, parent_job, created_at, updated_at, completed_at
                 FROM jobs ORDER BY created_at DESC LIMIT ?1",
                vec![limit.to_string()],
            ),
        };

        let mut stmt = conn.prepare(query)?;
        let params_refs: Vec<&dyn rusqlite::types::ToSql> = params
            .iter()
            .map(|p| p as &dyn rusqlite::types::ToSql)
            .collect();

        let rows = stmt
            .query_map(params_refs.as_slice(), |row| {
                Ok(Job {
                    id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                    pipeline: row.get(1)?,
                    project: row.get(2)?,
                    content_id: row
                        .get::<_, Option<String>>(3)?
                        .and_then(|s| Uuid::parse_str(&s).ok()),
                    step: row.get(4)?,
                    platform: row.get(5)?,
                    status: parse_job_status(&row.get::<_, String>(6)?),
                    payload: row
                        .get::<_, Option<String>>(7)?
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap_or(serde_json::json!({})),
                    result: row
                        .get::<_, Option<String>>(8)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    error: row.get(9)?,
                    attempts: row.get::<_, i32>(10)? as u32,
                    max_retries: row.get::<_, i32>(11)? as u32,
                    retry_after: row
                        .get::<_, Option<String>>(12)?
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&Utc)),
                    parent_job: row
                        .get::<_, Option<String>>(13)?
                        .and_then(|s| Uuid::parse_str(&s).ok()),
                    created_at: parse_dt(&row.get::<_, String>(14)?),
                    updated_at: parse_dt(&row.get::<_, String>(15)?),
                    completed_at: row.get::<_, Option<String>>(16)?.map(|s| parse_dt(&s)),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(rows)
    }
}

/// Exponential backoff: 30s, 60s, 120s, 240s, capped at 600s.
fn next_retry_delay(attempt: u32) -> chrono::Duration {
    let secs = std::cmp::min(30 * 2u64.pow(attempt.saturating_sub(1)), 600);
    chrono::Duration::seconds(secs as i64)
}

fn parse_job_status(s: &str) -> JobStatus {
    match s {
        "pending" => JobStatus::Pending,
        "running" => JobStatus::Running,
        "awaiting_review" => JobStatus::AwaitingReview,
        "approved" => JobStatus::Approved,
        "published" => JobStatus::Published,
        "failed" => JobStatus::Failed,
        "retrying" => JobStatus::Retrying,
        "cancelled" => JobStatus::Cancelled,
        "done" => JobStatus::Done,
        _ => JobStatus::Pending,
    }
}

fn parse_dt(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                .map(|dt| dt.and_utc())
                .unwrap_or_else(|_| Utc::now())
        })
}

/// Extension trait for optional query results.
trait OptionalExt<T> {
    fn optional(self) -> Result<Option<T>>;
}

impl<T> OptionalExt<T> for std::result::Result<T, rusqlite::Error> {
    fn optional(self) -> Result<Option<T>> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
