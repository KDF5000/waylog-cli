use crate::error::{Result, WaylogError};
use crate::providers::base::*;
use crate::utils::path;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tokio::fs;

pub struct CocoProvider;

impl CocoProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Provider for CocoProvider {
    fn name(&self) -> &str {
        "coco"
    }

    fn data_dir(&self) -> Result<PathBuf> {
        let home = path::home_dir()?;
        Ok(home.join(".cache").join("coco").join("sessions"))
    }

    fn session_dir(&self, _project_path: &Path) -> Result<PathBuf> {
        // Coco stores sessions in a flat directory structure, not per project
        // So we return the main sessions directory
        self.data_dir()
    }

    async fn find_latest_session(&self, project_path: &Path) -> Result<Option<PathBuf>> {
        let candidates = self.get_all_sessions(project_path).await?;
        Ok(candidates.into_iter().next())
    }

    async fn get_all_sessions(&self, project_path: &Path) -> Result<Vec<PathBuf>> {
        let session_dir = self.data_dir()?;

        if !session_dir.exists() {
            return Ok(Vec::new());
        }

        let mut entries = fs::read_dir(&session_dir).await?;
        let mut candidates = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                // Check session.json for cwd match
                let session_json_path = path.join("session.json");
                if session_json_path.exists() {
                    if let Ok(content) = fs::read_to_string(&session_json_path).await {
                        if let Ok(session_data) = serde_json::from_str::<CocoSessionData>(&content)
                        {
                            // Normalize paths for comparison
                            let session_cwd = std::fs::canonicalize(&session_data.metadata.cwd)
                                .unwrap_or_else(|_| session_data.metadata.cwd.clone());

                            let target_cwd = std::fs::canonicalize(project_path)
                                .unwrap_or_else(|_| project_path.to_path_buf());

                            if session_cwd == target_cwd {
                                // Found a match, use events.jsonl as the session file
                                let events_path = path.join("events.jsonl");
                                if events_path.exists() {
                                    // Parse updated_at to sort
                                    let updated_at =
                                        DateTime::parse_from_rfc3339(&session_data.updated_at)
                                            .map(|dt| dt.with_timezone(&Utc))
                                            .unwrap_or_else(|_| Utc::now());

                                    candidates.push((events_path, updated_at));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Sort by updated_at, newest first
        candidates.sort_by(|a, b| b.1.cmp(&a.1));

        Ok(candidates.into_iter().map(|(p, _)| p).collect())
    }

    async fn parse_session(&self, file_path: &Path) -> Result<ChatSession> {
        // file_path is events.jsonl
        // session.json is in the same directory
        let session_dir = file_path.parent().ok_or_else(|| {
            WaylogError::PathError("Could not find parent directory of session file".to_string())
        })?;

        let session_json_path = session_dir.join("session.json");
        let session_content = fs::read_to_string(&session_json_path).await?;
        let session_data: CocoSessionData =
            serde_json::from_str(&session_content).map_err(WaylogError::Json)?;

        let events_content = fs::read_to_string(file_path).await?;
        let mut messages = Vec::new();

        for line in events_content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            if let Ok(event) = serde_json::from_str::<CocoEventLine>(line) {
                if let Some(msg) = self.parse_event(event) {
                    messages.push(msg);
                }
            }
        }

        let started_at = DateTime::parse_from_rfc3339(&session_data.created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let updated_at = DateTime::parse_from_rfc3339(&session_data.updated_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(started_at);

        Ok(ChatSession {
            session_id: session_data.id,
            provider: self.name().to_string(),
            project_path: session_data.metadata.cwd,
            started_at,
            updated_at,
            messages,
        })
    }

    fn is_installed(&self) -> bool {
        self.data_dir().map(|d| d.exists()).unwrap_or(false)
    }

    fn command(&self) -> &str {
        "coco" // Assumed command name
    }
}

impl CocoProvider {
    fn parse_event(&self, event: CocoEventLine) -> Option<ChatMessage> {
        let (role, content) = if let Some(agent_start) = event.agent_start {
            // User input
            if let Some(first_input) = agent_start.input.first() {
                (MessageRole::User, first_input.content.clone())
            } else {
                return None;
            }
        } else if let Some(msg_event) = event.message {
            // Assistant output
            let role = match msg_event.message.role.as_str() {
                "user" => MessageRole::User,
                "assistant" => MessageRole::Assistant,
                _ => return None,
            };
            (role, msg_event.message.content)
        } else {
            return None;
        };

        if content.is_empty() {
            return None;
        }

        let timestamp = DateTime::parse_from_rfc3339(&event.created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Some(ChatMessage {
            id: event.id,
            timestamp,
            role,
            content,
            metadata: MessageMetadata::default(),
        })
    }
}

// Coco JSON structures

#[derive(Debug, Deserialize)]
struct CocoSessionData {
    id: String,
    created_at: String,
    updated_at: String,
    metadata: CocoSessionMetadata,
}

#[derive(Debug, Deserialize)]
struct CocoSessionMetadata {
    cwd: PathBuf,
}

#[derive(Debug, Deserialize)]
struct CocoEventLine {
    id: String,
    created_at: String,
    agent_start: Option<CocoAgentStart>,
    message: Option<CocoMessageEvent>,
}

#[derive(Debug, Deserialize)]
struct CocoAgentStart {
    input: Vec<CocoMessageContent>,
}

#[derive(Debug, Deserialize)]
struct CocoMessageEvent {
    message: CocoMessageContent,
}

#[derive(Debug, Deserialize)]
struct CocoMessageContent {
    role: String,
    content: String,
}
