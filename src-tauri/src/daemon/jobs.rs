use crate::ai::chunking::chunk_text;
use crate::ai::embeddings::{get_embedding_provider, EmbeddingProvider};
use crate::db::repositories::{documents as docs_repo, jobs as jobs_repo, meetings as meetings_repo, tasks as tasks_repo};
use crate::db::repositories::notifications as notif_repo;
use crate::governance::{approval as gov_approval, repository as gov_repo};
use crate::integrations::{get_provider, repository as integration_repo};
use crate::patterns::models::{
    AssigneePattern, CommunicationStyleModelData, PriorityPattern,
    SmartDefaultsModelData, UpsertPatternModelInput, WorkflowSequence, WorkflowSequenceModelData,
};
use crate::patterns::repository as patterns_repo;
use crate::skills::{
    self, approval, cron as skill_cron, repository as skills_repo, CreateSkillRunInput,
};
use crate::suggestions::models::CreateSuggestionInput;
use crate::suggestions::repository as suggestions_repo;
use crate::vectors::qdrant::{get_collection_name_with_dimension, QdrantClient, VectorPayload};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedDocumentPayload {
    pub document_id: String,
    pub project_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteSkillPayload {
    pub skill_id: String,
    pub skill_run_id: String,
    pub trigger_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncIntegrationPayload {
    pub integration_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    pub success: bool,
    pub chunks_embedded: Option<usize>,
    pub error: Option<String>,
}

pub struct JobContext {
    pub model_dir: PathBuf,
    pub qdrant: QdrantClient,
    pub embedding_provider: String,
    pub ollama_base_url: String,
    pub ollama_model: String,
}

pub async fn process_embed_document_job(
    conn: &Connection,
    job_id: &str,
    payload: &EmbedDocumentPayload,
    ctx: &JobContext,
) -> JobResult {
    // Mark job as running
    if let Err(e) = jobs_repo::update_job_status(conn, job_id, "running", None, None) {
        return JobResult {
            success: false,
            chunks_embedded: None,
            error: Some(format!("Failed to update job status: {}", e)),
        };
    }

    // Get document
    let doc = match docs_repo::get_document(conn, &payload.document_id) {
        Ok(Some(d)) => d,
        Ok(None) => {
            return JobResult {
                success: false,
                chunks_embedded: None,
                error: Some("Document not found".to_string()),
            };
        }
        Err(e) => {
            return JobResult {
                success: false,
                chunks_embedded: None,
                error: Some(format!("Failed to get document: {}", e)),
            };
        }
    };

    // Check if already embedded
    if doc.embeddings_ready {
        return JobResult {
            success: true,
            chunks_embedded: Some(0),
            error: None,
        };
    }

    // Get content text
    let content = match &doc.content_text {
        Some(c) if !c.is_empty() => c.clone(),
        _ => {
            return JobResult {
                success: false,
                chunks_embedded: None,
                error: Some("Document has no content to embed".to_string()),
            };
        }
    };

    // Get embedding provider
    let provider: Box<dyn EmbeddingProvider> = match get_embedding_provider(
        &ctx.embedding_provider,
        Some(ctx.model_dir.clone()),
        Some(&ctx.ollama_base_url),
        Some(&ctx.ollama_model),
    ) {
        Ok(p) => p,
        Err(e) => {
            return JobResult {
                success: false,
                chunks_embedded: None,
                error: Some(format!("Failed to get embedding provider: {}", e)),
            };
        }
    };

    // Chunk the content
    let chunks = chunk_text(&content, 500, 50);
    if chunks.is_empty() {
        return JobResult {
            success: false,
            chunks_embedded: None,
            error: Some("No chunks generated from content".to_string()),
        };
    }

    // Generate embeddings
    let mut vectors = Vec::new();
    for (idx, chunk_text) in chunks.iter().enumerate() {
        match provider.embed(chunk_text) {
            Ok(embedding) => {
                let point_id = format!("{}-{}", payload.document_id, idx);
                vectors.push((
                    point_id,
                    embedding,
                    VectorPayload {
                        document_id: payload.document_id.clone(),
                        chunk_index: idx as i32,
                        chunk_text: chunk_text.clone(),
                        project_id: Some(payload.project_id.clone()),
                    },
                ));
            }
            Err(e) => {
                return JobResult {
                    success: false,
                    chunks_embedded: Some(vectors.len()),
                    error: Some(format!("Failed to embed chunk {}: {}", idx, e)),
                };
            }
        }
    }

    // Store in Qdrant
    let collection = get_collection_name_with_dimension(
        Some(&payload.project_id),
        provider.dimensions(),
    );

    if let Err(e) = ctx.qdrant.insert_vectors(&collection, vectors.clone()).await {
        return JobResult {
            success: false,
            chunks_embedded: Some(vectors.len()),
            error: Some(format!("Failed to store vectors: {}", e)),
        };
    }

    // Update document status
    if let Err(e) = docs_repo::update_embeddings_ready(
        conn,
        &payload.document_id,
        true,
        provider.provider_name(),
    ) {
        return JobResult {
            success: false,
            chunks_embedded: Some(vectors.len()),
            error: Some(format!("Failed to update document status: {}", e)),
        };
    }

    JobResult {
        success: true,
        chunks_embedded: Some(vectors.len()),
        error: None,
    }
}

pub fn process_job_sync(
    conn: &Connection,
    job: &jobs_repo::DaemonJob,
    ctx: &JobContext,
) -> JobResult {
    match job.job_type.as_str() {
        "embed_document" => {
            let payload: EmbedDocumentPayload = match &job.payload {
                Some(p) => match serde_json::from_str(p) {
                    Ok(parsed) => parsed,
                    Err(e) => {
                        return JobResult {
                            success: false,
                            chunks_embedded: None,
                            error: Some(format!("Invalid job payload: {}", e)),
                        };
                    }
                },
                None => {
                    return JobResult {
                        success: false,
                        chunks_embedded: None,
                        error: Some("Missing job payload".to_string()),
                    };
                }
            };

            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(process_embed_document_job(conn, &job.id, &payload, ctx))
            })
        }
        "aggregate_patterns" => process_pattern_aggregation_job(conn, &job.id),
        "generate_suggestions" => process_suggestion_generation_job(conn, &job.id),
        "execute_skill" => {
            let payload: ExecuteSkillPayload = match &job.payload {
                Some(p) => match serde_json::from_str(p) {
                    Ok(parsed) => parsed,
                    Err(e) => {
                        return JobResult {
                            success: false,
                            chunks_embedded: None,
                            error: Some(format!("Invalid skill job payload: {}", e)),
                        };
                    }
                },
                None => {
                    return JobResult {
                        success: false,
                        chunks_embedded: None,
                        error: Some("Missing skill job payload".to_string()),
                    };
                }
            };
            process_execute_skill_job(conn, &job.id, &payload)
        }
        "poll_scheduled_skills" => process_poll_scheduled_skills_job(conn, &job.id),
        "check_skill_approvals" => process_check_skill_approvals_job(conn, &job.id),
        "sync_integration" => {
            let payload: SyncIntegrationPayload = match &job.payload {
                Some(p) => match serde_json::from_str(p) {
                    Ok(parsed) => parsed,
                    Err(e) => {
                        return JobResult {
                            success: false,
                            chunks_embedded: None,
                            error: Some(format!("Invalid sync job payload: {}", e)),
                        };
                    }
                },
                None => {
                    return JobResult {
                        success: false,
                        chunks_embedded: None,
                        error: Some("Missing sync job payload".to_string()),
                    };
                }
            };
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(process_sync_integration_job(conn, &job.id, &payload))
            })
        }
        "poll_integration_syncs" => process_poll_integration_syncs_job(conn, &job.id),
        "check_approval_timeouts" => process_check_approval_timeouts_job(conn, &job.id),
        "aggregate_governance_metrics" => process_aggregate_governance_metrics_job(conn, &job.id),
        "detect_anomalies" => process_detect_anomalies_job(conn, &job.id),
        _ => JobResult {
            success: false,
            chunks_embedded: None,
            error: Some(format!("Unknown job type: {}", job.job_type)),
        },
    }
}

pub fn process_pattern_aggregation_job(conn: &Connection, job_id: &str) -> JobResult {
    if let Err(e) = jobs_repo::update_job_status(conn, job_id, "running", None, None) {
        return JobResult {
            success: false,
            chunks_embedded: None,
            error: Some(format!("Failed to update job status: {}", e)),
        };
    }

    let observations = match patterns_repo::get_unprocessed_observations(conn, 1000, 0) {
        Ok(obs) => obs,
        Err(e) => {
            return JobResult {
                success: false,
                chunks_embedded: None,
                error: Some(format!("Failed to get observations: {}", e)),
            };
        }
    };

    if observations.is_empty() {
        let _ = patterns_repo::apply_pattern_decay(conn, 0.1, 30);
        schedule_next_aggregation(conn);
        return JobResult {
            success: true,
            chunks_embedded: Some(0),
            error: None,
        };
    }

    let mut task_completions: Vec<_> = vec![];
    let mut priority_sets: Vec<_> = vec![];
    let mut assignee_sets: Vec<_> = vec![];
    let mut draft_edits: Vec<_> = vec![];
    let mut processed_ids = vec![];

    for obs in &observations {
        processed_ids.push(obs.id.clone());
        match obs.observation_type.as_str() {
            "task_completion" => task_completions.push(obs),
            "priority_set" => priority_sets.push(obs),
            "assignee_set" => assignee_sets.push(obs),
            "draft_edit" => draft_edits.push(obs),
            _ => {}
        }
    }

    if let Err(e) = aggregate_workflow_sequences(conn, &task_completions) {
        eprintln!("Warning: workflow sequence aggregation failed: {}", e);
    }

    if let Err(e) = aggregate_smart_defaults(conn, &priority_sets, &assignee_sets) {
        eprintln!("Warning: smart defaults aggregation failed: {}", e);
    }

    if let Err(e) = aggregate_communication_style(conn, &draft_edits) {
        eprintln!("Warning: communication style aggregation failed: {}", e);
    }

    let _ = patterns_repo::mark_observations_processed(conn, &processed_ids);
    let _ = patterns_repo::apply_pattern_decay(conn, 0.1, 30);
    let _ = patterns_repo::prune_old_observations(conn, 90);

    schedule_next_aggregation(conn);

    JobResult {
        success: true,
        chunks_embedded: Some(processed_ids.len()),
        error: None,
    }
}

fn schedule_next_aggregation(conn: &Connection) {
    let _ = jobs_repo::create_job(conn, "aggregate_patterns", None, 1);
}

pub fn process_suggestion_generation_job(conn: &Connection, job_id: &str) -> JobResult {
    if let Err(e) = jobs_repo::update_job_status(conn, job_id, "running", None, None) {
        return JobResult {
            success: false,
            chunks_embedded: None,
            error: Some(format!("Failed to update job status: {}", e)),
        };
    }

    let daily_limit = get_suggestion_daily_limit(conn);
    let current_count = suggestions_repo::get_suggestions_count_today(conn).unwrap_or(0);

    if current_count >= daily_limit {
        schedule_next_suggestion_job(conn);
        return JobResult {
            success: true,
            chunks_embedded: Some(0),
            error: None,
        };
    }

    let remaining = (daily_limit - current_count) as usize;
    let mut suggestions_created = 0;

    if suggestions_created < remaining {
        suggestions_created += detect_overdue_tasks(conn, remaining - suggestions_created);
    }

    if suggestions_created < remaining {
        suggestions_created += detect_stale_tasks(conn, remaining - suggestions_created);
    }

    if suggestions_created < remaining {
        suggestions_created += detect_meeting_followups(conn, remaining - suggestions_created);
    }

    if suggestions_created < remaining {
        suggestions_created += detect_workflow_suggestions(conn, remaining - suggestions_created);
    }

    schedule_next_suggestion_job(conn);

    JobResult {
        success: true,
        chunks_embedded: Some(suggestions_created),
        error: None,
    }
}

fn schedule_next_suggestion_job(conn: &Connection) {
    let _ = jobs_repo::create_job(conn, "generate_suggestions", None, 1);
}

fn get_suggestion_daily_limit(conn: &Connection) -> i64 {
    conn.query_row(
        "SELECT value FROM app_settings WHERE key = 'suggestions_max_per_day'",
        [],
        |row| row.get::<_, String>(0),
    )
    .ok()
    .and_then(|s| s.parse::<i64>().ok())
    .unwrap_or(10)
}

fn detect_overdue_tasks(conn: &Connection, limit: usize) -> usize {
    let overdue_tasks = match tasks_repo::get_overdue_tasks(conn, 24) {
        Ok(tasks) => tasks,
        Err(_) => return 0,
    };

    let mut created = 0;
    for task in overdue_tasks.into_iter().take(limit) {
        let hours_overdue = calculate_hours_overdue(&task.due_date);
        let severity = if hours_overdue > 72 { "critical" } else if hours_overdue > 48 { "warning" } else { "info" };

        let _ = suggestions_repo::create_suggestion(
            conn,
            CreateSuggestionInput {
                suggestion_type: "overdue_task".to_string(),
                title: format!("Task overdue: {}", task.title),
                description: Some(format!("This task has been overdue for {} hours", hours_overdue)),
                reasoning: Some("Task due date has passed without completion".to_string()),
                action_config: Some(serde_json::json!({
                    "task_id": task.id,
                    "action": "focus"
                }).to_string()),
                severity: Some(severity.to_string()),
                project_id: Some(task.project_id.clone()),
            },
        );
        created += 1;
    }
    created
}

fn detect_stale_tasks(conn: &Connection, limit: usize) -> usize {
    let stale_tasks = match tasks_repo::get_stale_tasks(conn, 7) {
        Ok(tasks) => tasks,
        Err(_) => return 0,
    };

    let mut created = 0;
    for task in stale_tasks.into_iter().take(limit) {
        let _ = suggestions_repo::create_suggestion(
            conn,
            CreateSuggestionInput {
                suggestion_type: "stale_task".to_string(),
                title: format!("Stale task: {}", task.title),
                description: Some("This in-progress task hasn't been updated in over 7 days".to_string()),
                reasoning: Some("Long periods without updates may indicate blockers or deprioritization".to_string()),
                action_config: Some(serde_json::json!({
                    "task_id": task.id,
                    "action": "review"
                }).to_string()),
                severity: Some("info".to_string()),
                project_id: Some(task.project_id.clone()),
            },
        );
        created += 1;
    }
    created
}

fn detect_meeting_followups(conn: &Connection, limit: usize) -> usize {
    let orphan_meetings = match meetings_repo::get_meetings_without_tasks(conn, 24) {
        Ok(meetings) => meetings,
        Err(_) => return 0,
    };

    let mut created = 0;
    for meeting in orphan_meetings.into_iter().take(limit) {
        let _ = suggestions_repo::create_suggestion(
            conn,
            CreateSuggestionInput {
                suggestion_type: "meeting_followup".to_string(),
                title: format!("Follow up on: {}", meeting.title),
                description: Some("This meeting has no linked tasks. Consider extracting action items.".to_string()),
                reasoning: Some("Meetings older than 24 hours without tasks may have untracked action items".to_string()),
                action_config: Some(serde_json::json!({
                    "meeting_id": meeting.id,
                    "action": "extract_tasks"
                }).to_string()),
                severity: Some("info".to_string()),
                project_id: Some(meeting.project_id.clone()),
            },
        );
        created += 1;
    }
    created
}

fn detect_workflow_suggestions(conn: &Connection, limit: usize) -> usize {
    let recent_completions = match tasks_repo::get_recently_completed_tasks(conn, 24) {
        Ok(tasks) => tasks,
        Err(_) => return 0,
    };

    let mut created = 0;
    for task in recent_completions.into_iter().take(limit) {
        let project_id = &task.project_id;

        let workflow_model = match patterns_repo::get_pattern_model_by_type(conn, "workflow_sequence", Some(project_id)) {
            Ok(m) => m,
            Err(_) => continue,
        };

        if workflow_model.confidence < 0.5 {
            continue;
        }

        let model_data: crate::patterns::models::WorkflowSequenceModelData =
            match serde_json::from_str(&workflow_model.model_data) {
                Ok(d) => d,
                Err(_) => continue,
            };

        let task_keywords: Vec<&str> = task.title.split_whitespace()
            .filter(|w| w.len() > 3)
            .take(3)
            .collect();

        for seq in &model_data.sequences {
            if seq.occurrence_count < 3 {
                continue;
            }

            let trigger_words: Vec<&str> = seq.trigger_action.split_whitespace().collect();
            let matches = task_keywords.iter().any(|kw| trigger_words.iter().any(|tw| kw.eq_ignore_ascii_case(tw)));

            if matches {
                let _ = suggestions_repo::create_suggestion(
                    conn,
                    CreateSuggestionInput {
                        suggestion_type: "workflow_sequence".to_string(),
                        title: format!("Suggested next: {}", seq.follow_action),
                        description: Some(format!(
                            "Based on your workflow, '{}' is often followed by '{}'",
                            seq.trigger_action, seq.follow_action
                        )),
                        reasoning: Some(format!(
                            "This pattern has been observed {} times with {:.0}% confidence",
                            seq.occurrence_count, workflow_model.confidence * 100.0
                        )),
                        action_config: Some(serde_json::json!({
                            "action": "create_task",
                            "suggested_title": seq.follow_action,
                            "trigger_task_id": task.id
                        }).to_string()),
                        severity: Some("info".to_string()),
                        project_id: Some(task.project_id.clone()),
                    },
                );
                created += 1;
                break;
            }
        }
    }
    created
}

fn calculate_hours_overdue(due_date: &Option<String>) -> i64 {
    match due_date {
        Some(date_str) => {
            let now = chrono::Utc::now();
            match chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S") {
                Ok(due) => {
                    let due_utc = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(due, chrono::Utc);
                    let duration = now.signed_duration_since(due_utc);
                    duration.num_hours().max(0)
                }
                Err(_) => {
                    match chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                        Ok(due_date) => {
                            let due = due_date.and_hms_opt(23, 59, 59).unwrap();
                            let due_utc = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(due, chrono::Utc);
                            let duration = now.signed_duration_since(due_utc);
                            duration.num_hours().max(0)
                        }
                        Err(_) => 0,
                    }
                }
            }
        }
        None => 0,
    }
}

fn aggregate_workflow_sequences(
    conn: &Connection,
    completions: &[&crate::patterns::models::PatternObservation],
) -> Result<(), String> {
    let mut project_sequences: HashMap<String, Vec<WorkflowSequence>> = HashMap::new();

    for obs in completions {
        let project_id = match &obs.project_id {
            Some(p) => p.clone(),
            None => continue,
        };

        let context: serde_json::Value = serde_json::from_str(&obs.context_data)
            .map_err(|e| format!("Failed to parse context: {}", e))?;

        let task_keywords = context
            .get("task_keywords")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default();

        if task_keywords.is_empty() {
            continue;
        }

        let sequences = project_sequences.entry(project_id).or_insert_with(Vec::new);

        let existing = sequences.iter_mut().find(|s| s.trigger_action == task_keywords);
        if let Some(seq) = existing {
            seq.occurrence_count += 1;
        } else if !sequences.is_empty() {
            let last = sequences.last().unwrap();
            sequences.push(WorkflowSequence {
                trigger_action: last.follow_action.clone(),
                follow_action: task_keywords,
                occurrence_count: 1,
                avg_delay_minutes: 5.0,
            });
        } else {
            sequences.push(WorkflowSequence {
                trigger_action: "start".to_string(),
                follow_action: task_keywords,
                occurrence_count: 1,
                avg_delay_minutes: 0.0,
            });
        }
    }

    for (project_id, sequences) in project_sequences {
        let existing_model = patterns_repo::get_pattern_model_by_type(conn, "workflow_sequence", Some(&project_id));

        let (mut model_data, old_count) = match existing_model {
            Ok(m) => {
                let data: WorkflowSequenceModelData = serde_json::from_str(&m.model_data)
                    .unwrap_or(WorkflowSequenceModelData {
                        sequences: vec![],
                        negative_sequences: vec![],
                    });
                (data, m.observation_count)
            }
            Err(_) => (
                WorkflowSequenceModelData {
                    sequences: vec![],
                    negative_sequences: vec![],
                },
                0,
            ),
        };

        for new_seq in sequences {
            let existing = model_data
                .sequences
                .iter_mut()
                .find(|s| s.trigger_action == new_seq.trigger_action && s.follow_action == new_seq.follow_action);

            if let Some(seq) = existing {
                seq.occurrence_count += new_seq.occurrence_count;
            } else {
                model_data.sequences.push(new_seq);
            }
        }

        let total_count = old_count + completions.len() as i64;
        let confidence = calculate_confidence(total_count, &model_data.sequences);

        let _ = patterns_repo::upsert_pattern_model(
            conn,
            UpsertPatternModelInput {
                pattern_type: "workflow_sequence".to_string(),
                project_id: Some(project_id),
                model_data: serde_json::to_value(&model_data).unwrap(),
                confidence,
                observation_count: total_count,
            },
        );
    }

    Ok(())
}

fn aggregate_smart_defaults(
    conn: &Connection,
    priority_sets: &[&crate::patterns::models::PatternObservation],
    assignee_sets: &[&crate::patterns::models::PatternObservation],
) -> Result<(), String> {
    let mut project_data: HashMap<String, (Vec<PriorityPattern>, Vec<AssigneePattern>)> = HashMap::new();

    for obs in priority_sets {
        let project_id = match &obs.project_id {
            Some(p) => p.clone(),
            None => continue,
        };

        let context: serde_json::Value = serde_json::from_str(&obs.context_data)
            .map_err(|e| format!("Failed to parse context: {}", e))?;

        let new_priority = context.get("new_priority").and_then(|v| v.as_str()).unwrap_or("");
        let keywords = context
            .get("task_keywords")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_default();

        let (priority_patterns, _) = project_data.entry(project_id).or_insert_with(|| (vec![], vec![]));

        for keyword in keywords {
            let existing = priority_patterns.iter_mut().find(|p| p.keyword == keyword && p.priority == new_priority);
            if let Some(p) = existing {
                p.occurrence_count += 1;
            } else {
                priority_patterns.push(PriorityPattern {
                    keyword: keyword.to_string(),
                    priority: new_priority.to_string(),
                    occurrence_count: 1,
                });
            }
        }
    }

    for obs in assignee_sets {
        let project_id = match &obs.project_id {
            Some(p) => p.clone(),
            None => continue,
        };

        let context: serde_json::Value = serde_json::from_str(&obs.context_data)
            .map_err(|e| format!("Failed to parse context: {}", e))?;

        let new_assignee = context.get("new_assignee").and_then(|v| v.as_str()).unwrap_or("");
        let keywords = context
            .get("task_keywords")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_default();

        let (_, assignee_patterns) = project_data.entry(project_id).or_insert_with(|| (vec![], vec![]));

        for keyword in keywords {
            let existing = assignee_patterns.iter_mut().find(|p| p.keyword == keyword && p.assignee == new_assignee);
            if let Some(p) = existing {
                p.occurrence_count += 1;
            } else {
                assignee_patterns.push(AssigneePattern {
                    keyword: keyword.to_string(),
                    assignee: new_assignee.to_string(),
                    occurrence_count: 1,
                });
            }
        }
    }

    for (project_id, (priority_patterns, assignee_patterns)) in project_data {
        let existing_model = patterns_repo::get_pattern_model_by_type(conn, "smart_defaults", Some(&project_id));

        let (mut model_data, old_count) = match existing_model {
            Ok(m) => {
                let data: SmartDefaultsModelData = serde_json::from_str(&m.model_data)
                    .unwrap_or(SmartDefaultsModelData {
                        priority_patterns: vec![],
                        assignee_patterns: vec![],
                        project_defaults: HashMap::new(),
                    });
                (data, m.observation_count)
            }
            Err(_) => (
                SmartDefaultsModelData {
                    priority_patterns: vec![],
                    assignee_patterns: vec![],
                    project_defaults: HashMap::new(),
                },
                0,
            ),
        };

        for new_pattern in priority_patterns {
            let existing = model_data
                .priority_patterns
                .iter_mut()
                .find(|p| p.keyword == new_pattern.keyword && p.priority == new_pattern.priority);

            if let Some(p) = existing {
                p.occurrence_count += new_pattern.occurrence_count;
            } else {
                model_data.priority_patterns.push(new_pattern);
            }
        }

        for new_pattern in assignee_patterns {
            let existing = model_data
                .assignee_patterns
                .iter_mut()
                .find(|p| p.keyword == new_pattern.keyword && p.assignee == new_pattern.assignee);

            if let Some(p) = existing {
                p.occurrence_count += new_pattern.occurrence_count;
            } else {
                model_data.assignee_patterns.push(new_pattern);
            }
        }

        let total_count = old_count + (priority_sets.len() + assignee_sets.len()) as i64;
        let total_patterns = model_data.priority_patterns.len() + model_data.assignee_patterns.len();
        let confidence = if total_patterns > 0 {
            ((total_count as f64 / 10.0).min(1.0) * 0.8).min(1.0)
        } else {
            0.0
        };

        let _ = patterns_repo::upsert_pattern_model(
            conn,
            UpsertPatternModelInput {
                pattern_type: "smart_defaults".to_string(),
                project_id: Some(project_id),
                model_data: serde_json::to_value(&model_data).unwrap(),
                confidence,
                observation_count: total_count,
            },
        );
    }

    Ok(())
}

fn aggregate_communication_style(
    conn: &Connection,
    draft_edits: &[&crate::patterns::models::PatternObservation],
) -> Result<(), String> {
    if draft_edits.is_empty() {
        return Ok(());
    }

    let mut length_deltas: Vec<f64> = vec![];
    let mut additions: HashMap<String, i64> = HashMap::new();
    let mut removals: HashMap<String, i64> = HashMap::new();

    for obs in draft_edits {
        let context: serde_json::Value = serde_json::from_str(&obs.context_data)
            .map_err(|e| format!("Failed to parse context: {}", e))?;

        if let Some(delta) = context.get("length_delta").and_then(|v| v.as_f64()) {
            length_deltas.push(delta);
        }

        let original = context.get("original_text").and_then(|v| v.as_str()).unwrap_or("");
        let edited = context.get("edited_text").and_then(|v| v.as_str()).unwrap_or("");

        let original_words: std::collections::HashSet<_> = original.split_whitespace().collect();
        let edited_words: std::collections::HashSet<_> = edited.split_whitespace().collect();

        for word in edited_words.difference(&original_words) {
            if word.len() > 2 {
                *additions.entry(word.to_string()).or_insert(0) += 1;
            }
        }

        for word in original_words.difference(&edited_words) {
            if word.len() > 2 {
                *removals.entry(word.to_string()).or_insert(0) += 1;
            }
        }
    }

    let avg_length_delta = if !length_deltas.is_empty() {
        length_deltas.iter().sum::<f64>() / length_deltas.len() as f64
    } else {
        0.0
    };

    let length_preference = if avg_length_delta < -0.2 {
        "concise"
    } else if avg_length_delta > 0.2 {
        "verbose"
    } else {
        "neutral"
    };

    let mut common_additions: Vec<(String, i64)> = additions.into_iter().filter(|(_, c)| *c >= 2).collect();
    common_additions.sort_by(|a, b| b.1.cmp(&a.1));
    common_additions.truncate(10);

    let mut common_removals: Vec<(String, i64)> = removals.into_iter().filter(|(_, c)| *c >= 2).collect();
    common_removals.sort_by(|a, b| b.1.cmp(&a.1));
    common_removals.truncate(10);

    let existing_model = patterns_repo::get_pattern_model_by_type(conn, "communication_style", None);

    let (old_data, old_count) = match existing_model {
        Ok(m) => {
            let data: CommunicationStyleModelData = serde_json::from_str(&m.model_data)
                .unwrap_or(CommunicationStyleModelData {
                    length_preference: "neutral".to_string(),
                    formality_level: "neutral".to_string(),
                    common_additions: vec![],
                    common_removals: vec![],
                    signature_patterns: vec![],
                });
            (data, m.observation_count)
        }
        Err(_) => (
            CommunicationStyleModelData {
                length_preference: "neutral".to_string(),
                formality_level: "neutral".to_string(),
                common_additions: vec![],
                common_removals: vec![],
                signature_patterns: vec![],
            },
            0,
        ),
    };

    for (phrase, _) in &old_data.common_additions {
        if let Some(idx) = common_additions.iter().position(|(p, _)| p == phrase) {
            let old_count = old_data.common_additions.iter().find(|(p, _)| p == phrase).map(|(_, c)| *c).unwrap_or(0);
            common_additions[idx].1 += old_count;
        }
    }

    let model_data = CommunicationStyleModelData {
        length_preference: length_preference.to_string(),
        formality_level: old_data.formality_level,
        common_additions,
        common_removals,
        signature_patterns: old_data.signature_patterns,
    };

    let total_count = old_count + draft_edits.len() as i64;
    let confidence = ((total_count as f64 / 10.0).min(1.0) * 0.9).min(1.0);

    let _ = patterns_repo::upsert_pattern_model(
        conn,
        UpsertPatternModelInput {
            pattern_type: "communication_style".to_string(),
            project_id: None,
            model_data: serde_json::to_value(&model_data).unwrap(),
            confidence,
            observation_count: total_count,
        },
    );

    Ok(())
}

fn calculate_confidence(observation_count: i64, sequences: &[WorkflowSequence]) -> f64 {
    let base_confidence = (observation_count as f64 / 10.0).min(1.0);

    let consistency = if !sequences.is_empty() {
        let total_occurrences: i64 = sequences.iter().map(|s| s.occurrence_count).sum();
        let max_occurrence = sequences.iter().map(|s| s.occurrence_count).max().unwrap_or(0);
        if total_occurrences > 0 {
            max_occurrence as f64 / total_occurrences as f64
        } else {
            0.0
        }
    } else {
        0.0
    };

    (base_confidence * 0.6 + consistency * 0.4).min(1.0)
}

// ─── Skill Execution Jobs ────────────────────────────────────────────────────

pub fn process_execute_skill_job(
    conn: &Connection,
    job_id: &str,
    payload: &ExecuteSkillPayload,
) -> JobResult {
    if let Err(e) = jobs_repo::update_job_status(conn, job_id, "running", None, None) {
        return JobResult {
            success: false,
            chunks_embedded: None,
            error: Some(format!("Failed to update job status: {}", e)),
        };
    }

    // Get the skill
    let skill = match skills_repo::get_skill(conn, &payload.skill_id) {
        Ok(s) => s,
        Err(e) => {
            return JobResult {
                success: false,
                chunks_embedded: None,
                error: Some(format!("Failed to get skill: {}", e)),
            };
        }
    };

    // Get the run
    let run = match skills_repo::get_skill_run(conn, &payload.skill_run_id) {
        Ok(r) => r,
        Err(e) => {
            return JobResult {
                success: false,
                chunks_embedded: None,
                error: Some(format!("Failed to get skill run: {}", e)),
            };
        }
    };

    // Execute the skill
    let result = match skills::execute_skill(conn, &skill, &run) {
        Ok(r) => r,
        Err(e) => {
            let _ = skills::fail_skill_run(conn, &run.id, &e);
            return JobResult {
                success: false,
                chunks_embedded: None,
                error: Some(format!("Skill execution failed: {}", e)),
            };
        }
    };

    // Complete the run
    if let Err(e) = skills::complete_skill_run(conn, &run.id, &result) {
        return JobResult {
            success: false,
            chunks_embedded: None,
            error: Some(format!("Failed to complete skill run: {}", e)),
        };
    }

    // If it needs approval, create notification
    if result.needs_approval {
        if let Some(ref changes) = result.pending_changes {
            let _ = approval::create_approval_notification(conn, &skill, &run, changes);
        }
    }

    // Update next_run_at for scheduled skills
    if skill.trigger_type == "schedule" {
        if let Some(trigger_config) = skill.get_trigger_config() {
            if let Some(ref cron_expr) = trigger_config.cron {
                let timezone = trigger_config.timezone.as_deref();
                if let Ok(next_run) = skill_cron::compute_next_run(cron_expr, timezone) {
                    let _ = skills_repo::update_next_run_at(conn, &skill.id, &next_run);
                }
            }
        }
    }

    JobResult {
        success: true,
        chunks_embedded: None,
        error: None,
    }
}

pub fn process_poll_scheduled_skills_job(conn: &Connection, job_id: &str) -> JobResult {
    if let Err(e) = jobs_repo::update_job_status(conn, job_id, "running", None, None) {
        return JobResult {
            success: false,
            chunks_embedded: None,
            error: Some(format!("Failed to update job status: {}", e)),
        };
    }

    // Get all due scheduled skills
    let due_skills = match skills_repo::get_due_scheduled_skills(conn) {
        Ok(skills) => skills,
        Err(e) => {
            schedule_next_skill_poll(conn);
            return JobResult {
                success: false,
                chunks_embedded: None,
                error: Some(format!("Failed to get due skills: {}", e)),
            };
        }
    };

    let mut queued = 0;
    for skill in due_skills {
        // Create a skill run
        let run = match skills_repo::create_skill_run(
            conn,
            &CreateSkillRunInput {
                skill_id: skill.id.clone(),
                trigger_type: "schedule".to_string(),
                trigger_context: Some(serde_json::json!({
                    "scheduled_at": chrono::Utc::now().to_rfc3339(),
                })),
            },
        ) {
            Ok(r) => r,
            Err(_) => continue,
        };

        // Queue the execution job
        let payload = ExecuteSkillPayload {
            skill_id: skill.id.clone(),
            skill_run_id: run.id.clone(),
            trigger_type: "schedule".to_string(),
        };

        let _ = jobs_repo::create_job(
            conn,
            "execute_skill",
            Some(&serde_json::to_string(&payload).unwrap_or_default()),
            5, // Normal priority
        );

        queued += 1;
    }

    schedule_next_skill_poll(conn);

    JobResult {
        success: true,
        chunks_embedded: Some(queued),
        error: None,
    }
}

pub fn process_check_skill_approvals_job(conn: &Connection, job_id: &str) -> JobResult {
    if let Err(e) = jobs_repo::update_job_status(conn, job_id, "running", None, None) {
        return JobResult {
            success: false,
            chunks_embedded: None,
            error: Some(format!("Failed to update job status: {}", e)),
        };
    }

    // Check for expired approvals (24h timeout)
    let expired = match approval::check_expired_approvals(conn) {
        Ok(ids) => ids,
        Err(e) => {
            schedule_next_approval_check(conn);
            return JobResult {
                success: false,
                chunks_embedded: None,
                error: Some(format!("Failed to check expired approvals: {}", e)),
            };
        }
    };

    schedule_next_approval_check(conn);

    JobResult {
        success: true,
        chunks_embedded: Some(expired.len()),
        error: None,
    }
}

fn schedule_next_skill_poll(conn: &Connection) {
    let next_run = chrono::Utc::now() + chrono::Duration::seconds(60);
    let _ = jobs_repo::create_job_scheduled(
        conn,
        "poll_scheduled_skills",
        None,
        1, // Low priority
        &next_run.to_rfc3339(),
    );
}

fn schedule_next_approval_check(conn: &Connection) {
    let next_run = chrono::Utc::now() + chrono::Duration::minutes(15);
    let _ = jobs_repo::create_job_scheduled(
        conn,
        "check_skill_approvals",
        None,
        1, // Low priority
        &next_run.to_rfc3339(),
    );
}

// ─── Integration Sync Jobs ───────────────────────────────────────────────────

pub async fn process_sync_integration_job(
    conn: &Connection,
    job_id: &str,
    payload: &SyncIntegrationPayload,
) -> JobResult {
    if let Err(e) = jobs_repo::update_job_status(conn, job_id, "running", None, None) {
        return JobResult {
            success: false,
            chunks_embedded: None,
            error: Some(format!("Failed to update job status: {}", e)),
        };
    }

    // Get the integration
    let integration = match integration_repo::get_integration(conn, &payload.integration_id) {
        Ok(Some(i)) => i,
        Ok(None) => {
            return JobResult {
                success: false,
                chunks_embedded: None,
                error: Some("Integration not found".to_string()),
            };
        }
        Err(e) => {
            return JobResult {
                success: false,
                chunks_embedded: None,
                error: Some(format!("Failed to get integration: {}", e)),
            };
        }
    };

    // Get the provider
    let provider = match get_provider(&integration.integration_type) {
        Some(p) => p,
        None => {
            return JobResult {
                success: false,
                chunks_embedded: None,
                error: Some(format!("Unknown integration type: {}", integration.integration_type)),
            };
        }
    };

    // Update status to syncing
    let _ = integration_repo::update_integration_status(conn, &payload.integration_id, "syncing", None);

    // Fetch data from the provider
    let fetch_result = provider.fetch_data(&integration).await;

    match fetch_result {
        Ok(result) => {
            let mut items_synced = 0;
            for item in &result.items {
                if integration_repo::upsert_cache_item(
                    conn,
                    &payload.integration_id,
                    &item.external_type,
                    &item.external_id,
                    item.external_url.as_deref(),
                    &item.data,
                ).is_ok() {
                    items_synced += 1;
                }
            }

            // Update last sync time
            let _ = integration_repo::update_integration_last_sync(conn, &payload.integration_id);
            let _ = integration_repo::update_integration_status(conn, &payload.integration_id, "connected", None);

            // Create notification if there were new items
            if items_synced > 0 {
                let _ = notif_repo::create_notification_full(
                    conn,
                    "integration_sync",
                    &format!("{} sync complete", integration.name),
                    &format!("Synced {} items from {}", items_synced, integration.name),
                    None,
                    None,
                    None,
                    Some(&integration.id),
                    "info",
                    false,
                );
            }

            // Create notifications for errors
            if !result.errors.is_empty() {
                let _ = notif_repo::create_notification_full(
                    conn,
                    "integration_sync_error",
                    &format!("{} sync warnings", integration.name),
                    &format!("{} errors during sync", result.errors.len()),
                    None,
                    None,
                    None,
                    Some(&integration.id),
                    "warning",
                    false,
                );
            }

            JobResult {
                success: true,
                chunks_embedded: Some(items_synced),
                error: None,
            }
        }
        Err(e) => {
            let _ = integration_repo::update_integration_status(
                conn,
                &payload.integration_id,
                "error",
                Some(&e),
            );

            // Create error notification
            let _ = notif_repo::create_notification_full(
                conn,
                "integration_sync_error",
                &format!("{} sync failed", integration.name),
                &e,
                None,
                None,
                None,
                Some(&integration.id),
                "critical",
                true, // Desktop notification for errors
            );

            JobResult {
                success: false,
                chunks_embedded: None,
                error: Some(e),
            }
        }
    }
}

pub fn process_poll_integration_syncs_job(conn: &Connection, job_id: &str) -> JobResult {
    if let Err(e) = jobs_repo::update_job_status(conn, job_id, "running", None, None) {
        return JobResult {
            success: false,
            chunks_embedded: None,
            error: Some(format!("Failed to update job status: {}", e)),
        };
    }

    // Get all connected integrations
    let integrations = match integration_repo::list_integrations(conn) {
        Ok(list) => list,
        Err(e) => {
            schedule_next_integration_sync(conn);
            return JobResult {
                success: false,
                chunks_embedded: None,
                error: Some(format!("Failed to list integrations: {}", e)),
            };
        }
    };

    let mut queued = 0;
    let now = chrono::Utc::now();

    for integration in integrations {
        // Skip disconnected or errored integrations
        if integration.status != "connected" {
            continue;
        }

        // Google has no generic fetch_data content (no issues/PRs equivalent
        // for a directory of people) — its roster sync runs on its own daily
        // schedule via poll_team_syncs. Running it here too would just call
        // a no-op and stamp a fake "last synced" time with nothing behind it.
        if integration.integration_type == "google" {
            continue;
        }

        // Check if sync is due based on sync_interval_minutes
        let should_sync = match &integration.last_sync {
            Some(last_sync_str) => {
                match chrono::DateTime::parse_from_rfc3339(last_sync_str) {
                    Ok(last_sync) => {
                        let minutes_since = (now - last_sync.with_timezone(&chrono::Utc))
                            .num_minutes();
                        minutes_since >= integration.sync_interval_minutes as i64
                    }
                    Err(_) => true, // Invalid timestamp, sync anyway
                }
            }
            None => true, // Never synced, sync now
        };

        if should_sync {
            let payload = SyncIntegrationPayload {
                integration_id: integration.id.clone(),
            };

            if jobs_repo::create_job(
                conn,
                "sync_integration",
                Some(&serde_json::to_string(&payload).unwrap_or_default()),
                5, // Normal priority
            ).is_ok() {
                queued += 1;
            }
        }
    }

    schedule_next_integration_sync(conn);

    JobResult {
        success: true,
        chunks_embedded: Some(queued),
        error: None,
    }
}

fn schedule_next_integration_sync(conn: &Connection) {
    // Check every 5 minutes for integrations that need syncing
    let next_run = chrono::Utc::now() + chrono::Duration::minutes(5);
    let _ = jobs_repo::create_job_scheduled(
        conn,
        "poll_integration_syncs",
        None,
        1, // Low priority
        &next_run.to_rfc3339(),
    );
}

/// Initialize integration sync jobs on daemon startup
pub fn init_integration_jobs(conn: &Connection) {
    // Check if poll job already exists
    let pending_polls = jobs_repo::get_pending_jobs_by_type(conn, "poll_integration_syncs")
        .unwrap_or_default();
    if pending_polls.is_empty() {
        schedule_next_integration_sync(conn);
    }
}

/// Queue a skill for execution based on an event trigger
pub fn queue_skill_for_event(
    conn: &Connection,
    skill_id: &str,
    trigger_context: serde_json::Value,
) -> Result<String, String> {
    // Create a skill run
    let run = skills_repo::create_skill_run(
        conn,
        &CreateSkillRunInput {
            skill_id: skill_id.to_string(),
            trigger_type: "event".to_string(),
            trigger_context: Some(trigger_context),
        },
    )?;

    // Queue the execution job
    let payload = ExecuteSkillPayload {
        skill_id: skill_id.to_string(),
        skill_run_id: run.id.clone(),
        trigger_type: "event".to_string(),
    };

    jobs_repo::create_job(
        conn,
        "execute_skill",
        Some(&serde_json::to_string(&payload).map_err(|e| e.to_string())?),
        7, // Higher priority for events
    )?;

    Ok(run.id)
}

/// Initialize skill polling jobs on daemon startup
pub fn init_skill_jobs(conn: &Connection) {
    // Check if poll job already exists
    let pending_polls = jobs_repo::get_pending_jobs_by_type(conn, "poll_scheduled_skills")
        .unwrap_or_default();
    if pending_polls.is_empty() {
        schedule_next_skill_poll(conn);
    }

    // Check if approval check job already exists
    let pending_checks = jobs_repo::get_pending_jobs_by_type(conn, "check_skill_approvals")
        .unwrap_or_default();
    if pending_checks.is_empty() {
        schedule_next_approval_check(conn);
    }

    // Initialize integration sync jobs
    init_integration_jobs(conn);

    // Initialize governance jobs
    init_governance_jobs(conn);
}

// ─── Governance Jobs ────────────────────────────────────────────────────────

pub fn process_check_approval_timeouts_job(conn: &Connection, job_id: &str) -> JobResult {
    if let Err(e) = jobs_repo::update_job_status(conn, job_id, "running", None, None) {
        return JobResult {
            success: false,
            chunks_embedded: None,
            error: Some(format!("Failed to update job status: {}", e)),
        };
    }

    let archived = match gov_approval::archive_expired_approvals(conn) {
        Ok(ids) => ids,
        Err(e) => {
            schedule_next_governance_approval_check(conn);
            return JobResult {
                success: false,
                chunks_embedded: None,
                error: Some(format!("Failed to archive expired approvals: {}", e)),
            };
        }
    };

    if !archived.is_empty() {
        let _ = notif_repo::create_notification_full(
            conn,
            "approval_timeout",
            "Approvals timed out",
            &format!("{} pending approvals have expired and been archived", archived.len()),
            None,
            None,
            None,
            None,
            "warning",
            false,
        );
    }

    schedule_next_governance_approval_check(conn);

    JobResult {
        success: true,
        chunks_embedded: Some(archived.len()),
        error: None,
    }
}

pub fn process_aggregate_governance_metrics_job(conn: &Connection, job_id: &str) -> JobResult {
    if let Err(e) = jobs_repo::update_job_status(conn, job_id, "running", None, None) {
        return JobResult {
            success: false,
            chunks_embedded: None,
            error: Some(format!("Failed to update job status: {}", e)),
        };
    }

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    // Count actions by risk level
    let risk_counts = count_audit_actions_by_risk(conn, &today);
    for (risk_level, count) in &risk_counts {
        let _ = gov_repo::upsert_governance_metric(
            conn,
            &today,
            "risk_distribution",
            Some(risk_level),
            *count,
        );
    }

    // Count actions by autonomy mode
    let autonomy_counts = count_audit_actions_by_autonomy(conn, &today);
    for (mode, count) in &autonomy_counts {
        let _ = gov_repo::upsert_governance_metric(
            conn,
            &today,
            "autonomy_breakdown",
            Some(mode),
            *count,
        );
    }

    // Count approval outcomes
    let approval_counts = count_approval_outcomes(conn, &today);
    for (status, count) in &approval_counts {
        let _ = gov_repo::upsert_governance_metric(
            conn,
            &today,
            "approval_rate",
            Some(status),
            *count,
        );
    }

    // Total action count
    let total_actions: i64 = risk_counts.values().sum();
    let _ = gov_repo::upsert_governance_metric(
        conn,
        &today,
        "action_count",
        None,
        total_actions,
    );

    // Count actions by integration source
    let integration_counts = count_actions_by_integration(conn, &today);
    for (integration_name, count) in &integration_counts {
        let _ = gov_repo::upsert_governance_metric(
            conn,
            &today,
            "integration_activity",
            Some(integration_name),
            *count,
        );
    }

    // Count actions by skill source
    let skill_counts = count_actions_by_skill(conn, &today);
    for (skill_name, count) in &skill_counts {
        let _ = gov_repo::upsert_governance_metric(
            conn,
            &today,
            "skill_activity",
            Some(skill_name),
            *count,
        );
    }

    schedule_next_governance_metrics_aggregation(conn);

    JobResult {
        success: true,
        chunks_embedded: Some(risk_counts.len()),
        error: None,
    }
}

pub fn process_detect_anomalies_job(conn: &Connection, job_id: &str) -> JobResult {
    if let Err(e) = jobs_repo::update_job_status(conn, job_id, "running", None, None) {
        return JobResult {
            success: false,
            chunks_embedded: None,
            error: Some(format!("Failed to update job status: {}", e)),
        };
    }

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let seven_days_ago = (chrono::Utc::now() - chrono::Duration::days(7))
        .format("%Y-%m-%d")
        .to_string();

    // Get average daily action count
    let metrics = gov_repo::get_governance_metrics(conn, &seven_days_ago, &today, Some("action_count"))
        .unwrap_or_default();

    if metrics.len() < 3 {
        schedule_next_anomaly_detection(conn);
        return JobResult {
            success: true,
            chunks_embedded: Some(0),
            error: None,
        };
    }

    let avg_actions: f64 = metrics.iter().map(|m| m.value as f64).sum::<f64>() / metrics.len() as f64;
    let today_actions = metrics.iter()
        .find(|m| m.date == today)
        .map(|m| m.value)
        .unwrap_or(0);

    let mut anomalies_detected = 0;

    // Check for activity spike (>2x average)
    if today_actions as f64 > avg_actions * 2.0 && avg_actions > 5.0 {
        let _ = notif_repo::create_notification_full(
            conn,
            "anomaly_detected",
            "Unusual activity detected",
            &format!(
                "Agent actions today ({}) are more than 2x the average ({})",
                today_actions,
                avg_actions as i64
            ),
            None,
            None,
            None,
            None,
            "warning",
            true,
        );
        anomalies_detected += 1;
    }

    // Check for high rejection rate
    let approval_metrics = gov_repo::get_governance_metrics(conn, &today, &today, Some("approval_rate"))
        .unwrap_or_default();

    let approved = approval_metrics.iter()
        .find(|m| m.breakdown_key.as_deref() == Some("approved"))
        .map(|m| m.value)
        .unwrap_or(0);
    let rejected = approval_metrics.iter()
        .find(|m| m.breakdown_key.as_deref() == Some("rejected"))
        .map(|m| m.value)
        .unwrap_or(0);

    let total_resolved = approved + rejected;
    if total_resolved > 5 && rejected as f64 / total_resolved as f64 > 0.5 {
        let _ = notif_repo::create_notification_full(
            conn,
            "anomaly_detected",
            "High rejection rate",
            &format!(
                "{:.0}% of agent actions were rejected today. Review agent suggestions.",
                (rejected as f64 / total_resolved as f64) * 100.0
            ),
            None,
            None,
            None,
            None,
            "warning",
            true,
        );
        anomalies_detected += 1;
    }

    schedule_next_anomaly_detection(conn);

    JobResult {
        success: true,
        chunks_embedded: Some(anomalies_detected),
        error: None,
    }
}

fn count_audit_actions_by_risk(conn: &Connection, date: &str) -> HashMap<String, i64> {
    let mut counts = HashMap::new();

    let query = format!(
        "SELECT risk_level, COUNT(*) FROM audit_log
         WHERE risk_level IS NOT NULL AND DATE(timestamp) = '{}'
         GROUP BY risk_level",
        date
    );

    if let Ok(mut stmt) = conn.prepare(&query) {
        let _ = stmt.query_map([], |row| {
            let risk_level: String = row.get(0)?;
            let count: i64 = row.get(1)?;
            counts.insert(risk_level, count);
            Ok(())
        }).map(|iter| iter.for_each(drop));
    }

    counts
}

fn count_audit_actions_by_autonomy(conn: &Connection, date: &str) -> HashMap<String, i64> {
    let mut counts = HashMap::new();

    let query = format!(
        "SELECT autonomy_mode, COUNT(*) FROM audit_log
         WHERE autonomy_mode IS NOT NULL AND DATE(timestamp) = '{}'
         GROUP BY autonomy_mode",
        date
    );

    if let Ok(mut stmt) = conn.prepare(&query) {
        let _ = stmt.query_map([], |row| {
            let mode: String = row.get(0)?;
            let count: i64 = row.get(1)?;
            counts.insert(mode, count);
            Ok(())
        }).map(|iter| iter.for_each(drop));
    }

    counts
}

fn count_approval_outcomes(conn: &Connection, date: &str) -> HashMap<String, i64> {
    let mut counts = HashMap::new();

    let query = format!(
        "SELECT status, COUNT(*) FROM pending_approvals
         WHERE DATE(resolved_at) = '{}' AND status IN ('approved', 'rejected', 'archived')
         GROUP BY status",
        date
    );

    if let Ok(mut stmt) = conn.prepare(&query) {
        let _ = stmt.query_map([], |row| {
            let status: String = row.get(0)?;
            let count: i64 = row.get(1)?;
            counts.insert(status, count);
            Ok(())
        }).map(|iter| iter.for_each(drop));
    }

    counts
}

fn count_actions_by_integration(conn: &Connection, date: &str) -> HashMap<String, i64> {
    let mut counts = HashMap::new();

    let query = format!(
        "SELECT COALESCE(i.name, pa.source_id), COUNT(*)
         FROM pending_approvals pa
         LEFT JOIN integrations i ON pa.source_id = i.id
         WHERE pa.source_type = 'integration' AND DATE(pa.created_at) = '{}'
         GROUP BY COALESCE(i.name, pa.source_id)",
        date
    );

    if let Ok(mut stmt) = conn.prepare(&query) {
        let _ = stmt.query_map([], |row| {
            let name: String = row.get(0)?;
            let count: i64 = row.get(1)?;
            counts.insert(name, count);
            Ok(())
        }).map(|iter| iter.for_each(drop));
    }

    counts
}

fn count_actions_by_skill(conn: &Connection, date: &str) -> HashMap<String, i64> {
    let mut counts = HashMap::new();

    let query = format!(
        "SELECT COALESCE(s.name, pa.source_id), COUNT(*)
         FROM pending_approvals pa
         LEFT JOIN skills s ON pa.source_id = s.id
         WHERE pa.source_type = 'skill' AND DATE(pa.created_at) = '{}'
         GROUP BY COALESCE(s.name, pa.source_id)",
        date
    );

    if let Ok(mut stmt) = conn.prepare(&query) {
        let _ = stmt.query_map([], |row| {
            let name: String = row.get(0)?;
            let count: i64 = row.get(1)?;
            counts.insert(name, count);
            Ok(())
        }).map(|iter| iter.for_each(drop));
    }

    counts
}

fn schedule_next_governance_approval_check(conn: &Connection) {
    let next_run = chrono::Utc::now() + chrono::Duration::minutes(1);
    let _ = jobs_repo::create_job_scheduled(
        conn,
        "check_approval_timeouts",
        None,
        1,
        &next_run.to_rfc3339(),
    );
}

fn schedule_next_governance_metrics_aggregation(conn: &Connection) {
    // Run daily at midnight
    let now = chrono::Utc::now();
    let tomorrow = (now + chrono::Duration::days(1))
        .date_naive()
        .and_hms_opt(0, 5, 0)
        .unwrap();
    let next_run = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
        tomorrow,
        chrono::Utc,
    );
    let _ = jobs_repo::create_job_scheduled(
        conn,
        "aggregate_governance_metrics",
        None,
        1,
        &next_run.to_rfc3339(),
    );
}

fn schedule_next_anomaly_detection(conn: &Connection) {
    let next_run = chrono::Utc::now() + chrono::Duration::hours(1);
    let _ = jobs_repo::create_job_scheduled(
        conn,
        "detect_anomalies",
        None,
        1,
        &next_run.to_rfc3339(),
    );
}

pub fn init_governance_jobs(conn: &Connection) {
    // Check for approval timeouts every minute
    let pending_timeouts = jobs_repo::get_pending_jobs_by_type(conn, "check_approval_timeouts")
        .unwrap_or_default();
    if pending_timeouts.is_empty() {
        schedule_next_governance_approval_check(conn);
    }

    // Aggregate metrics daily
    let pending_metrics = jobs_repo::get_pending_jobs_by_type(conn, "aggregate_governance_metrics")
        .unwrap_or_default();
    if pending_metrics.is_empty() {
        schedule_next_governance_metrics_aggregation(conn);
    }

    // Detect anomalies hourly
    let pending_anomalies = jobs_repo::get_pending_jobs_by_type(conn, "detect_anomalies")
        .unwrap_or_default();
    if pending_anomalies.is_empty() {
        schedule_next_anomaly_detection(conn);
    }

    // Initialize team sync jobs
    init_team_sync_jobs(conn);
}

// ─── Team Sync Jobs ─────────────────────────────────────────────────────────

use crate::team::repository as team_repo;
use crate::integrations::{google, slack};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncTeamRosterPayload {
    pub source: String, // "slack", "google"
    pub integration_id: Option<String>,
}

pub async fn process_sync_team_roster_job(
    conn: &Connection,
    job_id: &str,
    payload: &SyncTeamRosterPayload,
) -> JobResult {
    if let Err(e) = jobs_repo::update_job_status(conn, job_id, "running", None, None) {
        return JobResult {
            success: false,
            chunks_embedded: None,
            error: Some(format!("Failed to update job status: {}", e)),
        };
    }

    match payload.source.as_str() {
        "slack" => sync_team_from_slack_internal(conn).await,
        "google" => sync_team_from_google_internal(conn).await,
        _ => JobResult {
            success: false,
            chunks_embedded: None,
            error: Some(format!("Unknown team sync source: {}", payload.source)),
        },
    }
}

async fn sync_team_from_slack_internal(conn: &Connection) -> JobResult {
    // Get Slack integration
    let access_token = match integration_repo::get_integration_by_type(conn, "slack") {
        Ok(Some(integration)) => {
            match integration.config.access_token {
                Some(token) => token,
                None => {
                    return JobResult {
                        success: false,
                        chunks_embedded: None,
                        error: Some("No access token in Slack integration".to_string()),
                    };
                }
            }
        }
        Ok(None) => {
            return JobResult {
                success: true,
                chunks_embedded: Some(0),
                error: None, // No Slack integration, skip silently
            };
        }
        Err(e) => {
            return JobResult {
                success: false,
                chunks_embedded: None,
                error: Some(format!("Failed to get Slack integration: {}", e)),
            };
        }
    };

    // Fetch members from Slack
    let slack_members = match slack::fetch_workspace_members(&access_token).await {
        Ok(members) => members,
        Err(e) => {
            return JobResult {
                success: false,
                chunks_embedded: None,
                error: Some(format!("Failed to fetch Slack members: {}", e)),
            };
        }
    };

    let mut synced = 0;

    for member in &slack_members {
        let input = crate::team::models::CreateTeamMemberInput {
            name: member.name.clone(),
            email: member.email.clone(),
            avatar_url: member.avatar_url.clone(),
            source: "slack".to_string(),
            source_id: Some(member.id.clone()),
            role: None,
            expertise: None,
            metadata: None,
        };

        if team_repo::upsert_team_member(conn, &input).is_ok() {
            synced += 1;
        }
    }

    // Create notification if new members were synced
    if synced > 0 {
        let _ = notif_repo::create_notification_full(
            conn,
            "team_sync",
            "Team roster synced",
            &format!("Synced {} team members from Slack", synced),
            None,
            None,
            None,
            None,
            "info",
            false,
        );
    }

    JobResult {
        success: true,
        chunks_embedded: Some(synced),
        error: None,
    }
}

async fn sync_team_from_google_internal(conn: &Connection) -> JobResult {
    let (integration_id, access_token) = match integration_repo::get_integration_by_type(conn, "google") {
        Ok(Some(integration)) => match integration.config.access_token.clone() {
            Some(token) => (integration.id.clone(), token),
            None => {
                return JobResult {
                    success: false,
                    chunks_embedded: None,
                    error: Some("No access token in Google integration".to_string()),
                };
            }
        },
        Ok(None) => {
            return JobResult {
                success: true,
                chunks_embedded: Some(0),
                error: None, // No Google integration connected, skip silently
            };
        }
        Err(e) => {
            return JobResult {
                success: false,
                chunks_embedded: None,
                error: Some(format!("Failed to get Google integration: {}", e)),
            };
        }
    };

    let google_members = match google::fetch_workspace_members(&access_token).await {
        Ok(members) => members,
        Err(e) => {
            return JobResult {
                success: false,
                chunks_embedded: None,
                error: Some(format!("Failed to fetch Google Workspace members: {}", e)),
            };
        }
    };

    let mut synced = 0;

    for member in &google_members {
        let input = crate::team::models::CreateTeamMemberInput {
            name: member.name.clone(),
            email: member.email.clone(),
            avatar_url: member.avatar_url.clone(),
            source: "google".to_string(),
            source_id: Some(member.id.clone()),
            role: None,
            expertise: None,
            metadata: None,
        };

        if team_repo::upsert_team_member(conn, &input).is_ok() {
            synced += 1;
        }
    }

    // This is the ONLY thing that should stamp last_sync for Google — the
    // generic poll_integration_syncs job deliberately skips it (see
    // process_poll_integration_syncs_job) since it has no other content to sync.
    let _ = integration_repo::update_integration_last_sync(conn, &integration_id);

    if synced > 0 {
        let _ = notif_repo::create_notification_full(
            conn,
            "team_sync",
            "Team roster synced",
            &format!("Synced {} team members from Google Workspace", synced),
            None,
            None,
            None,
            None,
            "info",
            false,
        );
    }

    JobResult {
        success: true,
        chunks_embedded: Some(synced),
        error: None,
    }
}

pub fn process_poll_team_syncs_job(conn: &Connection, job_id: &str) -> JobResult {
    if let Err(e) = jobs_repo::update_job_status(conn, job_id, "running", None, None) {
        return JobResult {
            success: false,
            chunks_embedded: None,
            error: Some(format!("Failed to update job status: {}", e)),
        };
    }

    let mut queued = 0;

    // Check for Slack integration
    if let Ok(Some(_)) = integration_repo::get_integration_by_type(conn, "slack") {
        let payload = SyncTeamRosterPayload {
            source: "slack".to_string(),
            integration_id: None,
        };

        if jobs_repo::create_job(
            conn,
            "sync_team_roster",
            Some(&serde_json::to_string(&payload).unwrap_or_default()),
            3, // Lower priority than regular syncs
        ).is_ok() {
            queued += 1;
        }
    }

    // Check for Google integration
    if let Ok(Some(_)) = integration_repo::get_integration_by_type(conn, "google") {
        let payload = SyncTeamRosterPayload {
            source: "google".to_string(),
            integration_id: None,
        };

        if jobs_repo::create_job(
            conn,
            "sync_team_roster",
            Some(&serde_json::to_string(&payload).unwrap_or_default()),
            3,
        ).is_ok() {
            queued += 1;
        }
    }

    // Recompute workload scores for the whole roster alongside the daily sync.
    let _ = team_repo::compute_all_workload_scores(conn);

    schedule_next_team_sync(conn);

    JobResult {
        success: true,
        chunks_embedded: Some(queued),
        error: None,
    }
}

fn schedule_next_team_sync(conn: &Connection) {
    // Sync team roster daily
    let now = chrono::Utc::now();
    let tomorrow = (now + chrono::Duration::days(1))
        .date_naive()
        .and_hms_opt(2, 0, 0) // 2 AM UTC
        .unwrap();
    let next_run = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
        tomorrow,
        chrono::Utc,
    );
    let _ = jobs_repo::create_job_scheduled(
        conn,
        "poll_team_syncs",
        None,
        1,
        &next_run.to_rfc3339(),
    );
}

pub fn init_team_sync_jobs(conn: &Connection) {
    let pending_syncs = jobs_repo::get_pending_jobs_by_type(conn, "poll_team_syncs")
        .unwrap_or_default();
    if pending_syncs.is_empty() {
        schedule_next_team_sync(conn);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_scoring_low_observations() {
        let sequences = vec![WorkflowSequence {
            trigger_action: "review".to_string(),
            follow_action: "merge".to_string(),
            occurrence_count: 3,
            avg_delay_minutes: 5.0,
        }];

        // With 3 observations, base confidence = 0.3
        // With single sequence, consistency = 1.0
        // Result = 0.3 * 0.6 + 1.0 * 0.4 = 0.18 + 0.4 = 0.58
        let confidence = calculate_confidence(3, &sequences);
        assert!((confidence - 0.58).abs() < 0.01);
    }

    #[test]
    fn test_confidence_scoring_high_observations() {
        let sequences = vec![WorkflowSequence {
            trigger_action: "review".to_string(),
            follow_action: "merge".to_string(),
            occurrence_count: 10,
            avg_delay_minutes: 5.0,
        }];

        // With 15 observations, base confidence = 1.0 (capped)
        // With single sequence, consistency = 1.0
        // Result = 1.0 * 0.6 + 1.0 * 0.4 = 1.0
        let confidence = calculate_confidence(15, &sequences);
        assert!((confidence - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_confidence_scoring_multiple_sequences() {
        let sequences = vec![
            WorkflowSequence {
                trigger_action: "review".to_string(),
                follow_action: "merge".to_string(),
                occurrence_count: 8,
                avg_delay_minutes: 5.0,
            },
            WorkflowSequence {
                trigger_action: "review".to_string(),
                follow_action: "comment".to_string(),
                occurrence_count: 2,
                avg_delay_minutes: 3.0,
            },
        ];

        // With 10 observations, base confidence = 1.0
        // Total occurrences = 10, max = 8, consistency = 0.8
        // Result = 1.0 * 0.6 + 0.8 * 0.4 = 0.6 + 0.32 = 0.92
        let confidence = calculate_confidence(10, &sequences);
        assert!((confidence - 0.92).abs() < 0.01);
    }

    #[test]
    fn test_confidence_scoring_empty_sequences() {
        let sequences: Vec<WorkflowSequence> = vec![];

        // With 5 observations, base confidence = 0.5
        // With no sequences, consistency = 0.0
        // Result = 0.5 * 0.6 + 0.0 * 0.4 = 0.3
        let confidence = calculate_confidence(5, &sequences);
        assert!((confidence - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_confidence_scoring_low_consistency() {
        let sequences = vec![
            WorkflowSequence {
                trigger_action: "a".to_string(),
                follow_action: "b".to_string(),
                occurrence_count: 2,
                avg_delay_minutes: 1.0,
            },
            WorkflowSequence {
                trigger_action: "a".to_string(),
                follow_action: "c".to_string(),
                occurrence_count: 2,
                avg_delay_minutes: 1.0,
            },
            WorkflowSequence {
                trigger_action: "a".to_string(),
                follow_action: "d".to_string(),
                occurrence_count: 2,
                avg_delay_minutes: 1.0,
            },
            WorkflowSequence {
                trigger_action: "a".to_string(),
                follow_action: "e".to_string(),
                occurrence_count: 2,
                avg_delay_minutes: 1.0,
            },
        ];

        // With 10 observations, base confidence = 1.0
        // Total occurrences = 8, max = 2, consistency = 0.25
        // Result = 1.0 * 0.6 + 0.25 * 0.4 = 0.6 + 0.1 = 0.7
        let confidence = calculate_confidence(10, &sequences);
        assert!((confidence - 0.7).abs() < 0.01);
    }
}
