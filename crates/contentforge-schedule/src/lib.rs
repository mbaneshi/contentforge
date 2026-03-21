use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use chrono::Utc;
use contentforge_core::{ScheduleEntry, ScheduleStatus};
use contentforge_db::DbPool;
use contentforge_publish::PublisherRegistry;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for the scheduler.
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// How often the scheduler checks for pending entries.
    pub tick_interval: Duration,
    /// Maximum number of retries for a failed publish.
    pub max_retries: u32,
    /// Delay between retries.
    pub retry_delay: Duration,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            tick_interval: Duration::from_secs(30),
            max_retries: 3,
            retry_delay: Duration::from_secs(60),
        }
    }
}

// ---------------------------------------------------------------------------
// Scheduler
// ---------------------------------------------------------------------------

/// The main scheduler that periodically checks for pending schedule entries
/// and publishes them when their scheduled_at time has passed.
pub struct Scheduler {
    pub config: SchedulerConfig,
    pub db: DbPool,
    pub publishers: Arc<PublisherRegistry>,
}

impl Scheduler {
    pub fn new(db: DbPool, publishers: Arc<PublisherRegistry>, config: SchedulerConfig) -> Self {
        Self {
            config,
            db,
            publishers,
        }
    }

    /// Run the scheduler loop. This blocks forever (or until the task is cancelled).
    pub async fn run(&self) -> Result<()> {
        tracing::info!(
            interval_secs = self.config.tick_interval.as_secs(),
            "Scheduler started"
        );

        loop {
            tokio::time::sleep(self.config.tick_interval).await;
            if let Err(e) = self.tick().await {
                tracing::error!(error = %e, "Scheduler tick failed");
            }
        }
    }

    /// A single scheduler tick: find pending entries and publish those that are due.
    async fn tick(&self) -> Result<()> {
        let pending = self.get_pending_entries()?;
        let now = Utc::now();

        for entry in pending {
            if entry.scheduled_at <= now {
                tracing::info!(
                    entry_id = %entry.id,
                    content_id = %entry.content_id,
                    platform = %entry.platform,
                    "Publishing scheduled entry"
                );
                self.process_entry(&entry).await;
            }
        }

        Ok(())
    }

    /// Process a single schedule entry: attempt to publish, handle errors.
    async fn process_entry(&self, entry: &ScheduleEntry) {
        // Mark as in progress
        if let Err(e) = self.update_status(entry.id, ScheduleStatus::InProgress) {
            tracing::error!(error = %e, "Failed to update entry status");
            return;
        }

        // TODO: Load the content from DB, find the adaptation, and publish
        // For now this is a stub
        match self.try_publish(entry).await {
            Ok(()) => {
                tracing::info!(entry_id = %entry.id, "Successfully published");
                let _ = self.update_status(entry.id, ScheduleStatus::Published);
            }
            Err(e) => {
                tracing::error!(entry_id = %entry.id, error = %e, "Publish failed");
                if entry.retries < self.config.max_retries {
                    tracing::info!(
                        entry_id = %entry.id,
                        retry = entry.retries + 1,
                        "Will retry"
                    );
                    let _ = self.increment_retry(entry.id);
                } else {
                    tracing::error!(entry_id = %entry.id, "Max retries exceeded, marking as failed");
                    let _ = self.update_status(entry.id, ScheduleStatus::Failed);
                }
            }
        }
    }

    async fn try_publish(&self, _entry: &ScheduleEntry) -> Result<()> {
        // TODO: Look up content by entry.content_id, find adaptation for entry.platform,
        //       call publisher.publish(content, adaptation)
        todo!("Implement actual publish logic")
    }

    fn get_pending_entries(&self) -> Result<Vec<ScheduleEntry>> {
        // TODO: Query DB for entries with status = Pending
        todo!("Query pending schedule entries from DB")
    }

    fn update_status(&self, _id: uuid::Uuid, _status: ScheduleStatus) -> Result<()> {
        // TODO: Update schedule entry status in DB
        todo!("Update schedule entry status")
    }

    fn increment_retry(&self, _id: uuid::Uuid) -> Result<()> {
        // TODO: Increment retry count in DB
        todo!("Increment retry count")
    }
}
