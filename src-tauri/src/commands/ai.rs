use crate::ai::{embeddings, extractor, litellm::LiteLLMClient, ollama::OllamaClient};
use crate::db::repositories::{ai_settings as ai_repo, meetings as mtg_repo, projects as proj_repo, tasks as task_repo};
use crate::models::ai_settings::{AiSettings, AiSettingsInput, ModelInfo};
use crate::models::document::SearchResult;
use crate::AppState;
use serde_json::{json, Value};
use tauri::{Emitter, State};

/// Build a LiteLLMClient from saved AI settings. Used by other command modules.
pub fn get_litellm_client_pub(settings: &AiSettings, api_key: &str) -> LiteLLMClient {
    get_litellm_client(settings, api_key)
}

fn get_litellm_client(settings: &AiSettings, api_key: &str) -> LiteLLMClient {
    let base_url = settings
        .base_url
        .clone()
        .unwrap_or_else(|| match settings.provider.as_str() {
            "anthropic" => "https://api.anthropic.com/v1".to_string(),
            "gemini" => "https://generativelanguage.googleapis.com/v1beta/openai".to_string(),
            "groq" => "https://api.groq.com/openai/v1".to_string(),
            "litellm" => "http://localhost:4000".to_string(),
            _ => "https://api.openai.com/v1".to_string(),
        });
    let model = settings
        .model_id
        .clone()
        .filter(|m| !m.is_empty())
        .unwrap_or_else(|| match settings.provider.as_str() {
            "anthropic" => "claude-sonnet-4-6".to_string(),
            "gemini" => "gemini-pro".to_string(),
            "groq" => "llama-3.1-70b-versatile".to_string(),
            _ => "gpt-4o-mini".to_string(),
        });
    LiteLLMClient::new(base_url, api_key.to_string(), model)
}

/// Read the API key from app_settings (no Keychain — avoids macOS permission prompts).
/// Falls back to Keychain for users who saved their key before this change.
pub fn get_api_key_from_db(conn: &rusqlite::Connection, label: &str) -> String {
    let db_key = format!("ai_key_{}", label);
    conn.query_row(
        "SELECT value FROM app_settings WHERE key = ?1",
        rusqlite::params![db_key],
        |row| row.get::<_, String>(0),
    )
    .ok()
    // Keychain fallback for keys saved before this migration
    .or_else(|| {
        keyring::Entry::new("meridian", label)
            .ok()
            .and_then(|e| e.get_password().ok())
            .filter(|k| !k.is_empty())
    })
    .unwrap_or_default()
}

// Thin wrapper kept for the one caller (fetch_available_models) that has no DB access.
fn get_api_key(label: &str) -> String {
    keyring::Entry::new("meridian", label)
        .ok()
        .and_then(|e| e.get_password().ok())
        .unwrap_or_default()
}

#[tauri::command]
pub async fn verify_ai_connection(
    provider: String,
    base_url: Option<String>,
    api_key: String,
    model_id: Option<String>,
) -> Result<Value, String> {
    let url = base_url.unwrap_or_else(|| match provider.as_str() {
        "openai" => "https://api.openai.com/v1".to_string(),
        "anthropic" => "https://api.anthropic.com/v1".to_string(),
        "gemini" => "https://generativelanguage.googleapis.com/v1beta/openai".to_string(),
        "groq" => "https://api.groq.com/openai/v1".to_string(),
        _ => "http://localhost:4000".to_string(),
    });

    let has_model = model_id.as_ref().map_or(false, |m| !m.is_empty());

    let model = if has_model {
        model_id.unwrap()
    } else {
        // For providers without a known default, auto-detect from /models endpoint
        let known_default = match provider.as_str() {
            "openai" => Some("gpt-4o-mini".to_string()),
            "anthropic" => Some("claude-sonnet-4-6".to_string()),
            "gemini" => Some("gemini-pro".to_string()),
            "groq" => Some("llama-3.1-70b-versatile".to_string()),
            _ => None,
        };

        if let Some(default_model) = known_default {
            default_model
        } else {
            // Auto-detect: fetch /models and use the first available one
            let probe = LiteLLMClient::new(url.clone(), api_key.clone(), String::new());
            match probe.get_models().await {
                Ok(models) if !models.is_empty() => {
                    models[0]["id"].as_str().unwrap_or("gpt-4o-mini").to_string()
                }
                Ok(_) => {
                    return Ok(json!({
                        "success": false,
                        "error": "Connected to server but no models are available. Check your LiteLLM configuration.",
                        "latency_ms": 0,
                    }));
                }
                Err(e) => {
                    return Ok(json!({
                        "success": false,
                        "error": format!("Cannot reach server or list models: {}", e),
                        "latency_ms": 0,
                    }));
                }
            }
        }
    };

    let client = LiteLLMClient::new(url, api_key, model);

    match client.verify_connection().await {
        Ok((_, latency)) => Ok(json!({
            "success": true,
            "error": null,
            "latency_ms": latency,
        })),
        Err(e) => Ok(json!({
            "success": false,
            "error": e,
            "latency_ms": 0,
        })),
    }
}

#[tauri::command]
pub async fn fetch_available_models(
    provider: String,
    base_url: Option<String>,
    api_key_label: String,
    api_key: Option<String>,
) -> Result<Vec<ModelInfo>, String> {
    let api_key = api_key
        .filter(|k| !k.is_empty())
        .unwrap_or_else(|| get_api_key(&api_key_label));

    match provider.as_str() {
        "anthropic" => {
            // Return hardcoded Anthropic models
            Ok(vec![
                ModelInfo { id: "claude-opus-4-6".to_string(), name: "Claude Opus 4.6".to_string(), context_window: Some(200000) },
                ModelInfo { id: "claude-sonnet-4-6".to_string(), name: "Claude Sonnet 4.6".to_string(), context_window: Some(200000) },
                ModelInfo { id: "claude-haiku-4-5-20251001".to_string(), name: "Claude Haiku 4.5".to_string(), context_window: Some(200000) },
                ModelInfo { id: "claude-3-5-sonnet-20241022".to_string(), name: "Claude 3.5 Sonnet".to_string(), context_window: Some(200000) },
            ])
        }
        "ollama" => {
            let url = base_url.unwrap_or_else(|| "http://localhost:11434".to_string());
            let ollama = OllamaClient::new(url, "".to_string());
            let models = ollama.list_models().await?;
            Ok(models.into_iter().map(|m| ModelInfo {
                id: m.clone(),
                name: m,
                context_window: None,
            }).collect())
        }
        _ => {
            let url = base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string());
            let client = LiteLLMClient::new(url, api_key, "".to_string());
            let raw_models = client.get_models().await?;
            Ok(raw_models.into_iter().filter_map(|m| {
                let id = m["id"].as_str()?.to_string();
                Some(ModelInfo {
                    name: id.clone(),
                    id,
                    context_window: m["context_window"].as_u64().map(|v| v as u32),
                })
            }).collect())
        }
    }
}

#[tauri::command]
pub async fn save_ai_settings(
    settings: AiSettingsInput,
    state: State<'_, AppState>,
) -> Result<AiSettings, String> {
    let id = settings.id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Store API key in app_settings (no Keychain — avoids macOS permission prompts)
    if let Some(key) = &settings.api_key {
        if !key.is_empty() {
            let db_key = format!("ai_key_{}", settings.label);
            conn.execute(
                "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)",
                rusqlite::params![db_key, key],
            )
            .map_err(|e| format!("Failed to save API key: {}", e))?;
        }
    }
    ai_repo::save_settings(
        &conn,
        &id,
        &settings.label,
        &settings.provider,
        settings.base_url.as_deref(),
        settings.model_id.as_deref(),
        settings.ollama_base_url.as_deref().unwrap_or("http://localhost:11434"),
        settings.ollama_model.as_deref().unwrap_or("nomic-embed-text"),
        settings.embedding_provider.as_deref().unwrap_or("ollama"),
    )
}

#[tauri::command]
pub async fn get_ai_settings(state: State<'_, AppState>) -> Result<Option<AiSettings>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    ai_repo::get_active_settings(&conn)
}

#[tauri::command]
pub async fn extract_tasks_from_transcript(
    meeting_id: String,
    transcript: String,
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<crate::models::task::Task>, String> {
    if transcript.split_whitespace().count() < 50 {
        return Err("Transcript too short to extract tasks — paste the full meeting text".to_string());
    }

    let (settings, api_key, project, existing_titles, all_projects) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let settings = ai_repo::get_active_settings(&conn)?
            .ok_or_else(|| "No AI provider configured".to_string())?;
        let api_key = get_api_key_from_db(&conn, &settings.label);
        let project = proj_repo::get_project(&conn, &project_id)?
            .ok_or_else(|| "Project not found".to_string())?;
        let existing = task_repo::get_open_tasks_for_project(&conn, &project_id)?;
        let titles: Vec<String> = existing.iter().map(|t| t.title.clone()).collect();
        let all_projects = proj_repo::get_all_projects(&conn)?;
        (settings, api_key, project, titles, all_projects)
    };

    let all_project_names: Vec<String> = all_projects.iter().map(|p| p.name.clone()).collect();

    let litellm = get_litellm_client(&settings, &api_key);

    let extraction = extractor::extract_tasks(&litellm, &transcript, &project.name, &existing_titles, &all_project_names).await?;

    let mut inserted = vec![];
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    for task in &extraction.tasks {
        // Determine which project to create the task in.
        // If the AI set a "project" field and it matches an existing project (case-insensitive),
        // use that project instead of the current one.
        let target_project_id = if let Some(task_project_name) = &task.project {
            let matched = all_projects.iter().find(|p| {
                p.name.to_lowercase() == task_project_name.to_lowercase()
                    && p.id != project_id
            });
            matched.map(|p| p.id.clone()).unwrap_or_else(|| project_id.clone())
        } else {
            project_id.clone()
        };

        let input = crate::models::task::CreateTaskInput {
            project_id: target_project_id,
            meeting_id: Some(meeting_id.clone()),
            title: task.title.clone(),
            description: task.description.clone(),
            assignee: task.assignee.clone(),
            assignee_confidence: Some(task.assignee_confidence.clone()),
            assignee_source_quote: task.assignee_source_quote.clone(),
            due_date: task.due_date.clone(),
            due_confidence: Some(task.due_confidence.clone()),
            due_source_quote: task.due_source_quote.clone(),
            priority: task.priority.clone(),
            confidence_score: task.confidence_score,
            tags: Some(task.tags.clone()),
            kanban_column: None,
            notes: task.notes.clone(),
            is_duplicate: None,
            duplicate_of_id: None,
        };
        if let Ok(t) = task_repo::create_task(&conn, &input) {
            inserted.push(t);
        }
    }

    Ok(inserted)
}

#[tauri::command]
pub async fn chat_with_project(
    project_id: String,
    meeting_id: Option<String>,
    message: String,
    template_id: Option<String>,
    conversation_history: Option<Vec<Value>>,
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    let (settings, api_key, project, open_tasks, done_tasks, meetings, doc_results) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let settings = ai_repo::get_active_settings(&conn)?
            .ok_or_else(|| "No AI provider configured".to_string())?;
        let api_key = get_api_key_from_db(&conn, &settings.label);
        let project = crate::db::repositories::projects::get_project(&conn, &project_id)?
            .ok_or_else(|| "Project not found".to_string())?;

        let open_filters = crate::models::task::TaskFilters { status: Some("open".to_string()), ..Default::default() };
        let open_tasks = task_repo::get_tasks_for_project(&conn, &project_id, &open_filters)?;

        let done_filters = crate::models::task::TaskFilters { status: Some("done".to_string()), ..Default::default() };
        let done_tasks = task_repo::get_tasks_for_project(&conn, &project_id, &done_filters)?;

        let meetings = mtg_repo::get_meetings_for_project(&conn, &project_id)?;

        // FTS document search
        let doc_results: Vec<SearchResult> = if !message.is_empty() {
            search_documents_fts(&conn, &project_id, &message).unwrap_or_default()
        } else {
            vec![]
        };

        (settings, api_key, project, open_tasks, done_tasks, meetings, doc_results)
    };

    let litellm = get_litellm_client(&settings, &api_key);

    // Build context
    let context = extractor::build_project_context(
        &project.name,
        &open_tasks,
        &done_tasks,
        &meetings,
        &doc_results,
    );

    // Build system prompt
    let system_prompt = if let Some(tid) = &template_id {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        if let Some(template) = crate::db::repositories::prompt_templates::get_template(&conn, tid)? {
            let user_content = template.user_prompt_template
                .replace("{{project_name}}", &project.name)
                .replace("{{full_context}}", &context)
                .replace("{{user_question}}", &message)
                .replace("{{open_tasks}}", &format!("{} open tasks", open_tasks.len()))
                .replace("{{completed_tasks}}", &format!("{} completed tasks", done_tasks.len()))
                .replace("{{recent_meetings}}", &meetings.iter().take(3).map(|m| m.title.as_str()).collect::<Vec<_>>().join(", "));
            drop(conn);
            template.system_prompt + "\n\nContext:\n" + &context + "\n\n" + &user_content
        } else {
            drop(conn);
            crate::ai::prompts::CONTEXT_CHAT_SYSTEM.to_string() + "\n\nContext:\n" + &context
        }
    } else {
        crate::ai::prompts::CONTEXT_CHAT_SYSTEM.to_string() + "\n\nContext:\n" + &context
    };

    // Build messages
    let mut messages = vec![json!({"role": "system", "content": system_prompt})];
    if let Some(history) = conversation_history {
        messages.extend(history);
    }
    messages.push(json!({"role": "user", "content": message}));

    // Call AI (non-streaming for now — emit as single chunk)
    let response = litellm.chat_completion(messages, None).await?;

    // Emit as chat_chunk event
    let _ = app_handle.emit("chat_chunk", json!({"content": response, "done": true}));

    // Save to chat_history
    let msg_id = uuid::Uuid::new_v4().to_string();
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let user_id = uuid::Uuid::new_v4().to_string();
        let _ = conn.execute(
            "INSERT INTO chat_history (id, project_id, meeting_id, role, content, template_id) VALUES (?1,?2,?3,'user',?4,?5)",
            rusqlite::params![user_id, project_id, meeting_id, message, template_id],
        );
        let _ = conn.execute(
            "INSERT INTO chat_history (id, project_id, meeting_id, role, content, template_id) VALUES (?1,?2,?3,'assistant',?4,?5)",
            rusqlite::params![msg_id, project_id, meeting_id, response, template_id],
        );
    }

    Ok(json!({
        "id": msg_id,
        "role": "assistant",
        "content": response,
    }))
}

fn search_documents_fts(
    conn: &rusqlite::Connection,
    project_id: &str,
    query: &str,
) -> Result<Vec<SearchResult>, String> {
    // Sanitize query for FTS5: wrap each word in double quotes to avoid syntax errors
    let sanitized_query: String = query
        .split_whitespace()
        .filter(|w| !w.is_empty())
        .map(|w| format!("\"{}\"", w.replace('"', "")))
        .collect::<Vec<_>>()
        .join(" ");

    if sanitized_query.is_empty() {
        return Ok(vec![]);
    }

    let sql = "SELECT d.id, d.filename, df.content_text, COALESCE(d.title, d.filename)
               FROM documents_fts df
               JOIN documents d ON d.rowid = df.rowid
               WHERE documents_fts MATCH ?1 AND d.project_id = ?2
               LIMIT 5";

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let results = stmt
        .query_map(rusqlite::params![sanitized_query, project_id], |row| {
            let chunk_text: String = row.get(2)?;
            Ok(SearchResult {
                document_id: row.get(0)?,
                document_title: row.get(3)?,
                filename: row.get(1)?,
                content: chunk_text.clone(),
                chunk_text,
                score: 1.0,
                search_type: "keyword".to_string(),
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(results)
}

#[tauri::command]
pub async fn check_ollama_status(state: State<'_, AppState>) -> Result<Value, String> {
    let ollama_url = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        ai_repo::get_active_settings(&conn)?
            .and_then(|s| Some(s.ollama_base_url))
            .unwrap_or_else(|| "http://localhost:11434".to_string())
    };

    let ollama = OllamaClient::new(ollama_url, "nomic-embed-text".to_string());
    let (running, models) = ollama.check_status().await?;
    Ok(json!({ "running": running, "models": models }))
}

#[tauri::command]
pub async fn embed_document_chunks(
    document_id: String,
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    // Collect all data we need while holding the lock, then release before any await
    let (chunks, ollama_url, ollama_model) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let chunks_json: Option<String> = conn
            .query_row(
                "SELECT chunks FROM documents WHERE id = ?1",
                rusqlite::params![document_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        let chunks_json = chunks_json.ok_or_else(|| "Document has no chunks".to_string())?;
        let chunks: Vec<crate::models::document::DocumentChunk> =
            serde_json::from_str(&chunks_json).map_err(|e| e.to_string())?;
        let settings = ai_repo::get_active_settings(&conn)?
            .ok_or_else(|| "No AI settings configured".to_string())?;
        (chunks, settings.ollama_base_url, settings.ollama_model)
        // conn dropped here — no MutexGuard held across await
    };

    let total = chunks.len();

    // Embed all chunks asynchronously (no lock held)
    let ollama = OllamaClient::new(ollama_url.clone(), ollama_model.clone());
    // (id, chunk_index, chunk_text, embedding_bytes)
    let mut embedding_results: Vec<(String, i64, String, Vec<u8>)> = Vec::new();
    for (i, chunk) in chunks.iter().enumerate() {
        match ollama.embed(&chunk.text).await {
            Ok(embedding) => {
                let id = uuid::Uuid::new_v4().to_string();
                let bytes = embeddings::serialize_embedding(&embedding);
                embedding_results.push((id, chunk.index as i64, chunk.text.clone(), bytes));
            }
            Err(e) => eprintln!("Failed to embed chunk {}: {}", chunk.index, e),
        }
        let _ = app_handle.emit(
            "embed_progress",
            json!({
                "document_id": document_id,
                "progress": i + 1,
                "total": total,
            }),
        );
    }

    // Store results in a single lock acquisition
    let embedded_count = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        for (id, chunk_index, chunk_text, embedding_bytes) in &embedding_results {
            conn.execute(
                "INSERT INTO document_embeddings (id, document_id, chunk_index, chunk_text, embedding, model)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![id, document_id, chunk_index, chunk_text, embedding_bytes, ollama_model],
            )
            .map_err(|e| e.to_string())?;
        }
        conn.execute(
            "UPDATE documents SET embeddings_ready = 1, embedding_model = ?1 WHERE id = ?2",
            rusqlite::params![ollama_model, document_id],
        )
        .map_err(|e| e.to_string())?;
        embedding_results.len() as u32
    };

    Ok(json!({ "chunks_embedded": embedded_count }))
}

#[tauri::command]
pub async fn search_documents(
    project_id: String,
    query: String,
    use_semantic: bool,
    state: State<'_, AppState>,
) -> Result<Vec<SearchResult>, String> {
    // Collect all sync data while holding the lock, then release before any await
    let (mut results, semantic_prep) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let results = search_documents_fts(&conn, &project_id, &query).unwrap_or_default();

        let semantic_prep: Option<(String, String, Vec<(String, String, Vec<u8>, String)>)> =
            if use_semantic {
                let settings = ai_repo::get_active_settings(&conn)?;
                if let Some(s) = settings {
                    let sql = "SELECT de.id, de.document_id, de.chunk_text, de.embedding, d.filename
                               FROM document_embeddings de
                               JOIN documents d ON d.id = de.document_id
                               WHERE d.project_id = ?1";
                    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
                    let data: Vec<(String, String, Vec<u8>, String)> = stmt
                        .query_map(rusqlite::params![project_id], |row| {
                            Ok((
                                row.get::<_, String>(1)?,
                                row.get::<_, String>(2)?,
                                row.get::<_, Vec<u8>>(3)?,
                                row.get::<_, String>(4)?,
                            ))
                        })
                        .map_err(|e| e.to_string())?
                        .filter_map(|r| r.ok())
                        .collect();
                    Some((s.ollama_base_url, s.ollama_model, data))
                } else {
                    None
                }
            } else {
                None
            };
        (results, semantic_prep)
        // conn dropped here — no MutexGuard held across await
    };

    // Semantic search — no lock held
    if let Some((ollama_url, ollama_model, embeddings_data)) = semantic_prep {
        let ollama = OllamaClient::new(ollama_url, ollama_model);
        if let Ok(query_embedding) = ollama.embed(&query).await {
            let mut semantic_results: Vec<(f32, SearchResult)> = embeddings_data
                .into_iter()
                .map(|(doc_id, chunk_text, embedding_bytes, filename)| {
                    let embedding = embeddings::deserialize_embedding(&embedding_bytes);
                    let score = embeddings::cosine_similarity(&query_embedding, &embedding);
                    (score, SearchResult {
                        document_id: doc_id,
                        document_title: filename.clone(),
                        filename,
                        content: chunk_text.clone(),
                        chunk_text,
                        score: score as f64,
                        search_type: "semantic".to_string(),
                    })
                })
                .collect();

            semantic_results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
            results.extend(
                semantic_results.into_iter().take(5).map(|(_, r)| r),
            );
        }
    }

    results.dedup_by(|a, b| a.chunk_text == b.chunk_text);
    results.truncate(10);

    Ok(results)
}

#[tauri::command]
pub async fn generate_output(
    project_id: String,
    template_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let (settings, api_key, project, open_tasks, done_tasks, meetings, template) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let settings = ai_repo::get_active_settings(&conn)?
            .ok_or_else(|| "No AI provider configured".to_string())?;
        let api_key = get_api_key_from_db(&conn, &settings.label);
        let project = crate::db::repositories::projects::get_project(&conn, &project_id)?
            .ok_or_else(|| "Project not found".to_string())?;
        let open_filters = crate::models::task::TaskFilters { status: Some("open".to_string()), ..Default::default() };
        let open_tasks = task_repo::get_tasks_for_project(&conn, &project_id, &open_filters)?;
        let done_filters = crate::models::task::TaskFilters { status: Some("done".to_string()), ..Default::default() };
        let done_tasks = task_repo::get_tasks_for_project(&conn, &project_id, &done_filters)?;
        let meetings = mtg_repo::get_meetings_for_project(&conn, &project_id)?;
        let template = crate::db::repositories::prompt_templates::get_template(&conn, &template_id)?
            .ok_or_else(|| "Template not found".to_string())?;
        (settings, api_key, project, open_tasks, done_tasks, meetings, template)
    };

    let litellm = get_litellm_client(&settings, &api_key);

    let context = extractor::build_project_context(&project.name, &open_tasks, &done_tasks, &meetings, &[]);

    let user_content = template.user_prompt_template
        .replace("{{project_name}}", &project.name)
        .replace("{{full_context}}", &context)
        .replace("{{user_question}}", "")
        .replace("{{open_tasks}}", &format!("{} open tasks", open_tasks.len()))
        .replace("{{completed_tasks}}", &format!("{} completed tasks", done_tasks.len()))
        .replace(
            "{{recent_meetings}}",
            &meetings.iter().take(3).map(|m| m.title.as_str()).collect::<Vec<_>>().join(", "),
        );

    let messages = vec![
        serde_json::json!({"role": "system", "content": template.system_prompt}),
        serde_json::json!({"role": "user", "content": user_content}),
    ];

    litellm.chat_completion(messages, None).await
}
