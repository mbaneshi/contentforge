use anyhow::Result;
use contentforge_core::{ContentStatus, Platform, PlatformAdaptation};
use contentforge_db::repo::{AdaptationRepo, ContentRepo, PublicationRepo};
use contentforge_db::DbPool;
use contentforge_publish::PublisherRegistry;
use std::sync::Arc;
use std::time::Duration;

use crate::queue::{Job, JobQueue};

/// Pipeline worker — polls the job queue and executes steps.
pub struct Worker {
    queue: JobQueue,
    db: DbPool,
    publishers: Arc<PublisherRegistry>,
    poll_interval: Duration,
}

impl Worker {
    pub fn new(db: DbPool, publishers: Arc<PublisherRegistry>) -> Self {
        Self {
            queue: JobQueue::new(db.clone()),
            db,
            publishers,
            poll_interval: Duration::from_secs(5),
        }
    }

    /// Run the worker loop. This blocks forever (run in a tokio::spawn).
    pub async fn run(&self) {
        tracing::info!(
            "Pipeline worker started (poll interval: {:?})",
            self.poll_interval
        );

        loop {
            match self.queue.dequeue() {
                Ok(Some(job)) => {
                    tracing::info!(
                        job_id = %job.id,
                        pipeline = %job.pipeline,
                        step = %job.step,
                        attempt = job.attempts,
                        "Executing job"
                    );
                    self.execute_job(&job).await;
                }
                Ok(None) => {
                    // No jobs available, wait
                    tokio::time::sleep(self.poll_interval).await;
                }
                Err(e) => {
                    tracing::error!("Failed to dequeue job: {e}");
                    tokio::time::sleep(self.poll_interval).await;
                }
            }
        }
    }

    async fn execute_job(&self, job: &Job) {
        let result = match job.step.as_str() {
            "adapt" => self.step_adapt(job).await,
            "review" => self.step_review(job).await,
            "publish" => self.step_publish(job).await,
            "done" => {
                let _ = self
                    .queue
                    .complete(job.id, serde_json::json!({"status": "pipeline_complete"}));
                return;
            }
            other => {
                tracing::warn!(step = other, "Unknown step, marking complete");
                let _ = self.queue.complete(
                    job.id,
                    serde_json::json!({"status": "skipped", "reason": "unknown step"}),
                );
                return;
            }
        };

        match result {
            Ok(next_step) => {
                // Mark current job done
                let _ = self
                    .queue
                    .complete(job.id, serde_json::json!({"status": "completed"}));

                // Enqueue next step if there is one
                if let Some(step) = next_step {
                    let _ = self.queue.enqueue(
                        &job.pipeline,
                        &step,
                        job.content_id,
                        job.project.as_deref(),
                        job.platform.as_deref(),
                        job.payload.clone(),
                        Some(job.id),
                    );
                }
            }
            Err(e) => {
                let can_retry = !matches!(job.step.as_str(), "review");
                let _ = self.queue.fail(job.id, &e.to_string(), can_retry);
            }
        }
    }

    /// Adapt step: create platform adaptations for content.
    async fn step_adapt(&self, job: &Job) -> Result<Option<String>> {
        let content_id = job
            .content_id
            .ok_or_else(|| anyhow::anyhow!("Adapt step requires content_id"))?;

        let content_repo = ContentRepo::new(self.db.clone());
        let content = content_repo
            .get_by_id(content_id)?
            .ok_or_else(|| anyhow::anyhow!("Content not found"))?;

        // Get target platforms from payload or default
        let platforms: Vec<String> = job.payload["platforms"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_else(|| vec!["devto".to_string()]);

        let adapt_repo = AdaptationRepo::new(self.db.clone());

        for platform_str in &platforms {
            let platform: Platform = match platform_str.parse() {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!("Skipping unknown platform '{platform_str}': {e}");
                    continue;
                }
            };

            let adaptation = PlatformAdaptation {
                platform,
                title: Some(content.title.clone()),
                body: content.body.clone(),
                thread_parts: None,
                canonical_url: None,
                metadata: serde_json::json!({}),
            };

            adapt_repo.upsert(content_id, &adaptation)?;
            tracing::info!(platform = %platform, "Adapted content");
        }

        // Update content status
        content_repo.update_status(content_id, ContentStatus::Ready)?;

        // Next step: review (if pipeline includes it) or publish
        let skip_review = job.payload["skip_review"].as_bool().unwrap_or(false);
        if skip_review {
            Ok(Some("publish".to_string()))
        } else {
            Ok(Some("review".to_string()))
        }
    }

    /// Review step: suspend for human approval.
    async fn step_review(&self, job: &Job) -> Result<Option<String>> {
        self.queue.suspend_for_review(job.id)?;

        let short_id = &job.id.to_string()[..8];
        println!("\n=== Review Required ===");
        println!("Pipeline: {} | Step: review", job.pipeline);
        if let Some(ref project) = job.project {
            println!("Project: {project}");
        }
        println!("\nApprove:  contentforge pipeline approve {short_id}");
        println!("Reject:   contentforge pipeline reject {short_id} --reason \"...\"");
        println!("========================\n");

        // Don't enqueue next step — approval will do that
        Ok(None)
    }

    /// Publish step: publish to platforms.
    async fn step_publish(&self, job: &Job) -> Result<Option<String>> {
        let content_id = job
            .content_id
            .ok_or_else(|| anyhow::anyhow!("Publish step requires content_id"))?;

        let content_repo = ContentRepo::new(self.db.clone());
        let content = content_repo
            .get_by_id_full(content_id)?
            .ok_or_else(|| anyhow::anyhow!("Content not found"))?;

        if content.adaptations.is_empty() {
            return Err(anyhow::anyhow!("No adaptations found for content"));
        }

        let pub_repo = PublicationRepo::new(self.db.clone());
        let mut published = Vec::new();
        let mut errors = Vec::new();

        for adaptation in &content.adaptations {
            if let Some(publisher) = self.publishers.get(adaptation.platform) {
                match publisher.publish(&content, adaptation).await {
                    Ok(publication) => {
                        let _ = pub_repo.insert(&publication);
                        published.push(format!("{}: {}", adaptation.platform, publication.url));
                        tracing::info!(
                            platform = %adaptation.platform,
                            url = %publication.url,
                            "Published"
                        );
                    }
                    Err(e) => {
                        errors.push(format!("{}: {e}", adaptation.platform));
                        tracing::error!(platform = %adaptation.platform, error = %e, "Publish failed");
                    }
                }
            }
        }

        content_repo.update_status(content_id, ContentStatus::Published)?;

        if !errors.is_empty() && published.is_empty() {
            return Err(anyhow::anyhow!(
                "All publishes failed: {}",
                errors.join("; ")
            ));
        }

        self.queue.mark_published(
            job.id,
            serde_json::json!({
                "published": published,
                "errors": errors,
            }),
        )?;

        Ok(Some("done".to_string()))
    }
}
