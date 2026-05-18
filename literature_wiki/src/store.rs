use crate::types::{DocumentChunk, LiteratureDocument, LiteratureSummary};
use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct LiteratureStore {
    root: PathBuf,
}

impl LiteratureStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn from_env_or_default() -> Result<Self> {
        if let Ok(path) = std::env::var("LITERATURE_WIKI_WORKSPACE") {
            return Ok(Self::new(path));
        }

        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .context("HOME is not set; pass --workspace or set LITERATURE_WIKI_WORKSPACE")?;
        Ok(Self::new(home.join(".literature-wiki")))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn ensure(&self) -> Result<()> {
        fs::create_dir_all(self.documents_dir())?;
        fs::create_dir_all(self.index_dir())?;
        fs::create_dir_all(self.collections_dir())?;
        fs::create_dir_all(self.summaries_dir())?;
        Ok(())
    }

    pub fn documents_dir(&self) -> PathBuf {
        self.root.join("documents")
    }

    pub fn index_dir(&self) -> PathBuf {
        self.root.join("index")
    }

    pub fn collections_dir(&self) -> PathBuf {
        self.root.join("collections")
    }

    pub fn summaries_dir(&self) -> PathBuf {
        self.root.join("summaries")
    }

    pub fn document_dir(&self, document_id: &str) -> PathBuf {
        self.documents_dir().join(document_id)
    }

    pub fn metadata_path(&self, document_id: &str) -> PathBuf {
        self.document_dir(document_id).join("metadata.json")
    }

    pub fn text_path(&self, document_id: &str) -> PathBuf {
        self.document_dir(document_id).join("text.md")
    }

    pub fn chunks_path(&self, document_id: &str) -> PathBuf {
        self.document_dir(document_id).join("chunks.json")
    }

    pub fn global_chunks_path(&self) -> PathBuf {
        self.index_dir().join("chunks.json")
    }

    pub fn summary_path(&self, document_id: &str) -> PathBuf {
        self.summaries_dir().join(format!("{document_id}.json"))
    }

    pub fn write_json<T: Serialize>(&self, path: impl AsRef<Path>, value: &T) -> Result<()> {
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }
        let body = serde_json::to_string_pretty(value)?;
        fs::write(path, format!("{body}\n"))?;
        Ok(())
    }

    pub fn read_json<T: DeserializeOwned>(&self, path: impl AsRef<Path>) -> Result<T> {
        let body = fs::read_to_string(path.as_ref())
            .with_context(|| format!("failed to read {}", path.as_ref().display()))?;
        Ok(serde_json::from_str(&body)?)
    }

    pub fn save_document(&self, document: &LiteratureDocument) -> Result<()> {
        self.write_json(self.metadata_path(&document.document_id), document)
    }

    pub fn load_document(&self, document_id: &str) -> Result<LiteratureDocument> {
        self.read_json(self.metadata_path(document_id))
    }

    pub fn list_documents(&self) -> Result<Vec<LiteratureDocument>> {
        self.ensure()?;
        let mut docs: Vec<LiteratureDocument> = Vec::new();
        for entry in fs::read_dir(self.documents_dir())? {
            let entry = entry?;
            let path = entry.path().join("metadata.json");
            if path.is_file() {
                docs.push(self.read_json(path)?);
            }
        }
        docs.sort_by(|a, b| a.document_id.cmp(&b.document_id));
        Ok(docs)
    }

    pub fn save_chunks(&self, document_id: &str, chunks: &[DocumentChunk]) -> Result<()> {
        self.write_json(self.chunks_path(document_id), &chunks)
    }

    pub fn load_chunks(&self, document_id: &str) -> Result<Vec<DocumentChunk>> {
        let path = self.chunks_path(document_id);
        if !path.exists() {
            return Ok(Vec::new());
        }
        self.read_json(path)
    }

    pub fn save_global_chunks(&self, chunks: &[DocumentChunk]) -> Result<()> {
        self.write_json(self.global_chunks_path(), &chunks)
    }

    pub fn load_global_chunks(&self) -> Result<Vec<DocumentChunk>> {
        let path = self.global_chunks_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        self.read_json(path)
    }

    pub fn save_summary(&self, summary: &LiteratureSummary) -> Result<()> {
        self.write_json(self.summary_path(&summary.document_id), summary)
    }

    pub fn load_summary(&self, document_id: &str) -> Result<Option<LiteratureSummary>> {
        let path = self.summary_path(document_id);
        if !path.exists() {
            return Ok(None);
        }
        self.read_json(path).map(Some)
    }
}
