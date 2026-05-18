use crate::parser::{detect_format, parse_document};
use crate::store::LiteratureStore;
use crate::types::{AddDocumentRequest, AddDocumentResponse, LiteratureDocument};
use anyhow::{Context, Result};
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub fn add_document(
    store: &LiteratureStore,
    request: AddDocumentRequest,
) -> Result<AddDocumentResponse> {
    store.ensure()?;
    let source_path = request.path.canonicalize().with_context(|| {
        format!(
            "failed to resolve source document path {}",
            request.path.display()
        )
    })?;
    let format = detect_format(&source_path)?;
    let bytes = fs::read(&source_path)?;
    let content_sha256 = hex_sha256(&bytes);
    let document_id = format!("doc_{}", &content_sha256[..16]);
    let document_dir = store.document_dir(&document_id);
    let stored_path = document_dir.join(format!("original.{}", format.canonical_extension()));
    let text_path = store.text_path(&document_id);

    if store.metadata_path(&document_id).exists() {
        return Ok(AddDocumentResponse {
            document: store.load_document(&document_id)?,
            duplicate: true,
        });
    }

    fs::create_dir_all(&document_dir)?;
    fs::copy(&source_path, &stored_path)?;
    let text = parse_document(&source_path, &format)?;
    fs::write(&text_path, text)?;

    let document = LiteratureDocument {
        document_id,
        source_path,
        stored_path,
        text_path,
        format,
        metadata: request.metadata,
        imported_at: Utc::now(),
        content_sha256,
    };
    store.save_document(&document)?;
    Ok(AddDocumentResponse {
        document,
        duplicate: false,
    })
}

pub fn list_documents(store: &LiteratureStore) -> Result<Vec<LiteratureDocument>> {
    store.list_documents()
}

pub fn get_document(store: &LiteratureStore, document_id: &str) -> Result<LiteratureDocument> {
    store.load_document(document_id)
}

pub fn remove_document(store: &LiteratureStore, document_id: &str) -> Result<bool> {
    let path = store.document_dir(document_id);
    if !path.exists() {
        return Ok(false);
    }
    fs::remove_dir_all(path)?;
    let remaining_chunks = store
        .load_global_chunks()?
        .into_iter()
        .filter(|chunk| chunk.document_id != document_id)
        .collect::<Vec<_>>();
    store.save_global_chunks(&remaining_chunks)?;
    Ok(true)
}

fn hex_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn title_from_path(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| stem.replace(['_', '-'], " "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AddDocumentRequest, PaperMetadata};
    use tempfile::TempDir;

    #[test]
    fn imports_and_deduplicates_text_documents() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("paper.TXT");
        fs::write(&source, "retrieval augmented generation").unwrap();
        let store = LiteratureStore::new(temp.path().join("kb"));

        let first = add_document(
            &store,
            AddDocumentRequest {
                path: source.clone(),
                metadata: PaperMetadata::default(),
            },
        )
        .unwrap();
        let second = add_document(
            &store,
            AddDocumentRequest {
                path: source,
                metadata: PaperMetadata::default(),
            },
        )
        .unwrap();

        assert!(!first.duplicate);
        assert!(second.duplicate);
        assert_eq!(first.document.document_id, second.document.document_id);
        assert_eq!(list_documents(&store).unwrap().len(), 1);
    }
}
