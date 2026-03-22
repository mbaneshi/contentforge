-- Pipeline jobs table — the core of the automation engine.
CREATE TABLE jobs (
    id          TEXT PRIMARY KEY,
    pipeline    TEXT NOT NULL,     -- pipeline name: 'publish-all', 'adapt-and-review'
    project     TEXT,              -- associated project name
    content_id  TEXT REFERENCES content(id) ON DELETE SET NULL,
    step        TEXT NOT NULL,     -- current step: 'draft', 'adapt', 'review', 'publish', 'done'
    platform    TEXT,              -- target platform for platform-specific steps
    status      TEXT NOT NULL DEFAULT 'pending',
        -- pending | running | awaiting_review | approved | published | failed | retrying | cancelled
    payload     TEXT NOT NULL DEFAULT '{}',  -- JSON: inputs for this step
    result      TEXT,                        -- JSON: output of completed step
    error       TEXT,                        -- error message if failed
    attempts    INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 3,
    retry_after TEXT,              -- ISO 8601 timestamp for next retry
    parent_job  TEXT REFERENCES jobs(id) ON DELETE CASCADE,  -- for sub-jobs in a pipeline
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT
);

CREATE INDEX idx_jobs_status ON jobs(status);
CREATE INDEX idx_jobs_pipeline ON jobs(pipeline, project);
CREATE INDEX idx_jobs_retry ON jobs(status, retry_after);
CREATE INDEX idx_jobs_parent ON jobs(parent_job);
CREATE INDEX idx_jobs_content ON jobs(content_id);
