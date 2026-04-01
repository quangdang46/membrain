use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use xxhash_rust::xxh64::xxh64;

/// One markdown-defined operator workflow loaded from disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorWorkflow {
    pub runbook_id: String,
    pub title: String,
    pub source_doc: String,
    pub section: String,
    pub summary: String,
    pub steps: Vec<String>,
    pub definition_digest: String,
    pub updated_at_ms: u128,
}

/// One operator-facing workflow hint enriched with the loaded workflow definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorWorkflowHint {
    pub runbook_id: String,
    pub title: String,
    pub source_doc: String,
    pub section: String,
    pub summary: String,
    pub steps: Vec<String>,
    pub definition_digest: String,
    pub reason: String,
}

/// In-memory catalog of markdown-defined operator workflows.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OperatorWorkflowCatalog {
    workflows: BTreeMap<String, OperatorWorkflow>,
}

impl OperatorWorkflowCatalog {
    /// Loads workflow definitions from the repository-local default workflow directory.
    pub fn load_default() -> Result<Self, String> {
        Self::load_from_dir(default_workflow_dir())
    }

    /// Loads all workflow definitions from one directory.
    pub fn load_from_dir(dir: impl AsRef<Path>) -> Result<Self, String> {
        let dir = dir.as_ref();
        if !dir.exists() {
            return Err(format!(
                "workflow directory '{}' does not exist",
                dir.display()
            ));
        }

        let mut paths = fs::read_dir(dir)
            .map_err(|err| {
                format!(
                    "failed to read workflow directory '{}': {err}",
                    dir.display()
                )
            })?
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.extension().is_some_and(|ext| ext == "md"))
            .collect::<Vec<_>>();
        paths.sort();

        let mut workflows = BTreeMap::new();
        for path in paths {
            let workflow = load_workflow(&path, dir)?;
            if workflows
                .insert(workflow.runbook_id.clone(), workflow)
                .is_some()
            {
                return Err(format!(
                    "duplicate workflow id detected while loading '{}'",
                    path.display()
                ));
            }
        }

        Ok(Self { workflows })
    }

    /// Resolves one workflow hint, falling back to the provided metadata when the workflow is absent.
    pub fn hint_for(
        &self,
        runbook_id: &str,
        fallback_source_doc: &str,
        fallback_section: &str,
        reason: impl Into<String>,
    ) -> OperatorWorkflowHint {
        let reason = reason.into();
        if let Some(workflow) = self.workflows.get(runbook_id) {
            return OperatorWorkflowHint {
                runbook_id: workflow.runbook_id.clone(),
                title: workflow.title.clone(),
                source_doc: workflow.source_doc.clone(),
                section: workflow.section.clone(),
                summary: workflow.summary.clone(),
                steps: workflow.steps.clone(),
                definition_digest: workflow.definition_digest.clone(),
                reason,
            };
        }

        OperatorWorkflowHint {
            runbook_id: runbook_id.to_string(),
            title: runbook_id.replace('_', " "),
            source_doc: fallback_source_doc.to_string(),
            section: fallback_section.to_string(),
            summary: "fallback workflow definition".to_string(),
            steps: Vec::new(),
            definition_digest: "fallback".to_string(),
            reason,
        }
    }
}

fn default_workflow_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("docs/workflows")
}

fn load_workflow(path: &Path, base_dir: &Path) -> Result<OperatorWorkflow, String> {
    let content = fs::read_to_string(path)
        .map_err(|err| format!("failed to read workflow '{}': {err}", path.display()))?;
    let (metadata, body) = parse_frontmatter(&content)?;
    let runbook_id = metadata.get("id").cloned().ok_or_else(|| {
        format!(
            "workflow '{}' is missing required 'id' metadata",
            path.display()
        )
    })?;
    let title = metadata.get("title").cloned().ok_or_else(|| {
        format!(
            "workflow '{}' is missing required 'title' metadata",
            path.display()
        )
    })?;
    let summary = metadata.get("summary").cloned().ok_or_else(|| {
        format!(
            "workflow '{}' is missing required 'summary' metadata",
            path.display()
        )
    })?;
    let source_doc = metadata.get("source_doc").cloned().unwrap_or_else(|| {
        path.strip_prefix(base_dir.parent().unwrap_or(base_dir))
            .unwrap_or(path)
            .display()
            .to_string()
    });
    let section = metadata
        .get("section")
        .cloned()
        .unwrap_or_else(|| format!("# {title}"));
    let steps = parse_steps(body);
    let updated_at_ms = fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    let definition_digest = format!("{:016x}", xxh64(content.as_bytes(), 0));

    Ok(OperatorWorkflow {
        runbook_id,
        title,
        source_doc,
        section,
        summary,
        steps,
        definition_digest,
        updated_at_ms,
    })
}

fn parse_frontmatter(content: &str) -> Result<(BTreeMap<String, String>, &str), String> {
    let Some(frontmatter) = content.strip_prefix("---\n") else {
        return Err("workflow markdown must begin with frontmatter".to_string());
    };
    let Some((raw_metadata, body)) = frontmatter.split_once("\n---\n") else {
        return Err("workflow markdown must close frontmatter with '---'".to_string());
    };

    let mut metadata = BTreeMap::new();
    for line in raw_metadata.lines().filter(|line| !line.trim().is_empty()) {
        let Some((key, value)) = line.split_once(':') else {
            return Err(format!("invalid workflow metadata line '{line}'"));
        };
        metadata.insert(key.trim().to_string(), value.trim().to_string());
    }

    Ok((metadata, body))
}

fn parse_steps(body: &str) -> Vec<String> {
    let mut in_steps = false;
    let mut steps = Vec::new();

    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.eq_ignore_ascii_case("## Steps") || trimmed.eq_ignore_ascii_case("### Steps") {
            in_steps = true;
            continue;
        }
        if in_steps && trimmed.starts_with("## ") {
            break;
        }
        if in_steps && trimmed.starts_with("- ") {
            steps.push(trimmed.trim_start_matches("- ").to_string());
        }
    }

    steps
}

#[cfg(test)]
mod tests {
    use super::OperatorWorkflowCatalog;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("membrain-workflows-{unique}"))
    }

    #[test]
    fn loads_workflow_metadata_and_steps_from_markdown() -> Result<(), Box<dyn std::error::Error>> {
        let dir = unique_temp_dir();
        fs::create_dir_all(&dir)?;
        let workflow_path = dir.join("incident_response.md");
        fs::write(
            &workflow_path,
            r#"---
id: incident_response
title: Incident Response
source_doc: docs/workflows/incident_response.md
section: ## Incident Response
summary: Contain degraded serving and recover authoritative operator confidence.
---
# Incident Response

## Steps
- Confirm the degraded reason and current blast radius.
- Freeze irreversible actions until authoritative state is rechecked.
"#,
        )?;

        let catalog =
            OperatorWorkflowCatalog::load_from_dir(&dir).map_err(std::io::Error::other)?;
        let hint = catalog.hint_for(
            "incident_response",
            "docs/OPERATIONS.md",
            "## 7. Incident Response",
            "doctor requested the incident workflow",
        );

        assert_eq!(hint.runbook_id, "incident_response");
        assert_eq!(hint.title, "Incident Response");
        assert_eq!(hint.section, "## Incident Response");
        assert_eq!(hint.steps.len(), 2);
        assert_ne!(hint.definition_digest, "fallback");

        fs::remove_dir_all(&dir)?;
        Ok(())
    }

    #[test]
    fn reloads_changed_workflow_definitions_on_subsequent_loads(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let dir = unique_temp_dir();
        fs::create_dir_all(&dir)?;
        let workflow_path = dir.join("repair_backlog_growth.md");
        fs::write(
            &workflow_path,
            r#"---
id: repair_backlog_growth
title: Repair Backlog Growth
source_doc: docs/workflows/repair_backlog_growth.md
section: ## Repair Backlog Growth
summary: Drain repair backlog before declaring the runtime healthy.
---
# Repair Backlog Growth

## Steps
- Inspect queued repair jobs.
"#,
        )?;

        let first_catalog =
            OperatorWorkflowCatalog::load_from_dir(&dir).map_err(std::io::Error::other)?;
        let first_hint = first_catalog.hint_for(
            "repair_backlog_growth",
            "docs/FAILURE_PLAYBOOK.md",
            "## 9. Repair backlog growth",
            "doctor requested the repair workflow",
        );

        fs::write(
            &workflow_path,
            r#"---
id: repair_backlog_growth
title: Repair Backlog Growth
source_doc: docs/workflows/repair_backlog_growth.md
section: ## Repair Backlog Growth
summary: Drain and verify the repair backlog before declaring recovery.
---
# Repair Backlog Growth

## Steps
- Inspect queued repair jobs.
- Re-run parity verification after the queue is empty.
"#,
        )?;

        let second_catalog =
            OperatorWorkflowCatalog::load_from_dir(&dir).map_err(std::io::Error::other)?;
        let second_hint = second_catalog.hint_for(
            "repair_backlog_growth",
            "docs/FAILURE_PLAYBOOK.md",
            "## 9. Repair backlog growth",
            "doctor requested the repair workflow",
        );

        assert_ne!(first_hint.definition_digest, second_hint.definition_digest);
        assert_eq!(second_hint.steps.len(), 2);
        assert!(second_hint.summary.contains("verify"));

        fs::remove_dir_all(&dir)?;
        Ok(())
    }
}
