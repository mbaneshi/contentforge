use anyhow::Result;
use serde::{Deserialize, Serialize};

/// A pipeline definition loaded from TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineDefinition {
    pub pipeline: PipelineMeta,
    pub steps: Vec<StepDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineMeta {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepDefinition {
    pub name: String,
    pub runner: Option<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub parallel_over: Vec<String>,
    pub retry: Option<u32>,
}

impl PipelineDefinition {
    /// Parse a pipeline definition from TOML string.
    pub fn from_toml(toml_str: &str) -> Result<Self> {
        let def: PipelineDefinition = toml::from_str(toml_str)?;
        Ok(def)
    }

    /// Load from a file path.
    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_toml(&content)
    }

    /// Get the ordered list of step names.
    pub fn step_names(&self) -> Vec<&str> {
        self.steps.iter().map(|s| s.name.as_str()).collect()
    }

    /// Get the step after the given step name, or None if it's the last.
    pub fn next_step(&self, current: &str) -> Option<&str> {
        let mut found = false;
        for step in &self.steps {
            if found {
                return Some(&step.name);
            }
            if step.name == current {
                found = true;
            }
        }
        None
    }
}

/// Built-in pipeline definitions.
pub fn builtin_pipelines() -> Vec<PipelineDefinition> {
    vec![
        PipelineDefinition::from_toml(PUBLISH_ALL_TOML).unwrap(),
        PipelineDefinition::from_toml(ADAPT_AND_REVIEW_TOML).unwrap(),
    ]
}

const PUBLISH_ALL_TOML: &str = r#"
[pipeline]
name = "publish-all"
description = "Adapt content for all configured platforms and publish immediately (no review)"

[[steps]]
name = "adapt"
runner = "platform_api"

[[steps]]
name = "publish"
runner = "platform_api"
retry = 3

[[steps]]
name = "done"
"#;

const ADAPT_AND_REVIEW_TOML: &str = r#"
[pipeline]
name = "adapt-review-publish"
description = "Adapt content, wait for review/approval, then publish"

[[steps]]
name = "adapt"
runner = "platform_api"

[[steps]]
name = "review"
runner = "approval"
depends_on = ["adapt"]

[[steps]]
name = "publish"
runner = "platform_api"
depends_on = ["review"]
retry = 3

[[steps]]
name = "done"
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_publish_all() {
        let def = PipelineDefinition::from_toml(PUBLISH_ALL_TOML).unwrap();
        assert_eq!(def.pipeline.name, "publish-all");
        assert_eq!(def.steps.len(), 3);
        assert_eq!(def.step_names(), vec!["adapt", "publish", "done"]);
        assert_eq!(def.next_step("adapt"), Some("publish"));
        assert_eq!(def.next_step("done"), None);
    }

    #[test]
    fn test_parse_adapt_review() {
        let def = PipelineDefinition::from_toml(ADAPT_AND_REVIEW_TOML).unwrap();
        assert_eq!(def.pipeline.name, "adapt-review-publish");
        assert_eq!(def.steps.len(), 4);
        assert_eq!(def.next_step("review"), Some("publish"));
    }
}
