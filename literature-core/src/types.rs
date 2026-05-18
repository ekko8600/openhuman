use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DocumentFormat {
    Markdown,
    Text,
    Pdf,
    Html,
}

impl DocumentFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "md" | "markdown" => Some(Self::Markdown),
            "txt" | "text" => Some(Self::Text),
            "pdf" => Some(Self::Pdf),
            "html" | "htm" => Some(Self::Html),
            _ => None,
        }
    }

    pub fn canonical_extension(&self) -> &'static str {
        match self {
            Self::Markdown => "md",
            Self::Text => "txt",
            Self::Pdf => "pdf",
            Self::Html => "html",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PaperMetadata {
    pub title: Option<String>,
    pub authors: Vec<String>,
    pub year: Option<i32>,
    pub doi: Option<String>,
    pub arxiv_id: Option<String>,
    pub source_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiteratureDocument {
    pub document_id: String,
    pub source_path: PathBuf,
    pub stored_path: PathBuf,
    pub text_path: PathBuf,
    pub format: DocumentFormat,
    pub metadata: PaperMetadata,
    pub imported_at: DateTime<Utc>,
    pub content_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AddDocumentRequest {
    pub path: PathBuf,
    #[serde(default)]
    pub metadata: PaperMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AddDocumentResponse {
    pub document: LiteratureDocument,
    pub duplicate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentChunk {
    pub document_id: String,
    pub chunk_id: String,
    pub ordinal: usize,
    pub heading_path: Vec<String>,
    pub text: String,
    pub char_start: usize,
    pub char_end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BuildIndexRequest {
    pub document_id: Option<String>,
    #[serde(default = "default_chunk_chars")]
    pub chunk_chars: usize,
}

pub fn default_chunk_chars() -> usize {
    1800
}

impl Default for BuildIndexRequest {
    fn default() -> Self {
        Self {
            document_id: None,
            chunk_chars: default_chunk_chars(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BuildIndexResponse {
    pub indexed_documents: usize,
    pub chunk_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchRequest {
    pub query: String,
    #[serde(default = "default_search_limit")]
    pub limit: usize,
}

pub fn default_search_limit() -> usize {
    8
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResult {
    pub document_id: String,
    pub title: Option<String>,
    pub chunk_id: String,
    pub score: f64,
    pub text: String,
    pub citation: ChunkCitation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChunkCitation {
    pub section: Option<String>,
    pub ordinal: usize,
    pub source_path: PathBuf,
    pub doi: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ControllerSchema {
    pub method: String,
    pub description: String,
    pub required: Vec<String>,
    pub optional: Vec<String>,
}
