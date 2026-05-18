use crate::store::LiteratureStore;
use crate::types::{
    BuildIndexRequest, BuildIndexResponse, ChunkCitation, DocumentChunk, LiteratureDocument,
    SearchRequest, SearchResult,
};
use anyhow::{Context, Result};
use regex::Regex;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::fs;

pub fn build_index(
    store: &LiteratureStore,
    request: BuildIndexRequest,
) -> Result<BuildIndexResponse> {
    store.ensure()?;
    let documents = match request.document_id {
        Some(document_id) => vec![store.load_document(&document_id)?],
        None => store.list_documents()?,
    };

    let mut all_chunks = if documents.len() == 1 {
        store
            .load_global_chunks()?
            .into_iter()
            .filter(|chunk| chunk.document_id != documents[0].document_id)
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let mut indexed_documents = 0;
    let mut chunk_count = 0;
    for document in &documents {
        let text = fs::read_to_string(&document.text_path)
            .with_context(|| format!("failed to read {}", document.text_path.display()))?;
        let chunks = chunk_document(document, &text, request.chunk_chars.max(256));
        chunk_count += chunks.len();
        indexed_documents += 1;
        store.save_chunks(&document.document_id, &chunks)?;
        all_chunks.extend(chunks);
    }

    all_chunks.sort_by(|a, b| a.chunk_id.cmp(&b.chunk_id));
    store.save_global_chunks(&all_chunks)?;
    Ok(BuildIndexResponse {
        indexed_documents,
        chunk_count,
    })
}

pub fn search_index(store: &LiteratureStore, request: SearchRequest) -> Result<Vec<SearchResult>> {
    let query_terms = tokenize(&request.query);
    if query_terms.is_empty() {
        return Ok(Vec::new());
    }
    let chunks = store.load_global_chunks()?;
    let mut results = Vec::new();
    for chunk in chunks {
        let score = score_chunk(&chunk.text, &query_terms);
        if score <= 0.0 {
            continue;
        }
        let document = store.load_document(&chunk.document_id)?;
        results.push(to_result(chunk, document, score));
    }
    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.chunk_id.cmp(&b.chunk_id))
    });
    results.truncate(request.limit.max(1));
    Ok(results)
}

pub fn chunk_document(
    document: &LiteratureDocument,
    text: &str,
    chunk_chars: usize,
) -> Vec<DocumentChunk> {
    let sections = split_sections(text);
    let mut chunks = Vec::new();
    let mut ordinal = 0;
    for section in sections {
        let mut start = section.char_start;
        let mut buffer = String::new();
        for word in section.text.split_whitespace() {
            let separator = if buffer.is_empty() { "" } else { " " };
            if !buffer.is_empty() && buffer.len() + separator.len() + word.len() > chunk_chars {
                let end = start + buffer.len();
                chunks.push(make_chunk(
                    document,
                    ordinal,
                    &section.heading_path,
                    &buffer,
                    start,
                    end,
                ));
                ordinal += 1;
                start = end;
                buffer.clear();
            }
            buffer.push_str(separator);
            buffer.push_str(word);
        }
        if !buffer.trim().is_empty() {
            let end = start + buffer.len();
            chunks.push(make_chunk(
                document,
                ordinal,
                &section.heading_path,
                &buffer,
                start,
                end,
            ));
            ordinal += 1;
        }
    }
    chunks
}

fn make_chunk(
    document: &LiteratureDocument,
    ordinal: usize,
    heading_path: &[String],
    text: &str,
    char_start: usize,
    char_end: usize,
) -> DocumentChunk {
    DocumentChunk {
        document_id: document.document_id.clone(),
        chunk_id: format!("{}:chunk_{ordinal:05}", document.document_id),
        ordinal,
        heading_path: heading_path.to_vec(),
        text: text.trim().to_string(),
        char_start,
        char_end,
    }
}

#[derive(Debug)]
struct Section {
    heading_path: Vec<String>,
    text: String,
    char_start: usize,
}

fn split_sections(text: &str) -> Vec<Section> {
    let heading_re = Regex::new(r"^(#{1,6})\s+(.+)$").expect("valid heading regex");
    let mut heading_stack: Vec<String> = Vec::new();
    let mut sections = Vec::new();
    let mut current = String::new();
    let mut current_start = 0;
    let mut offset = 0;

    for line in text.lines() {
        if let Some(caps) = heading_re.captures(line.trim()) {
            if !current.trim().is_empty() {
                sections.push(Section {
                    heading_path: heading_stack.clone(),
                    text: current.clone(),
                    char_start: current_start,
                });
                current.clear();
            }
            let level = caps[1].len();
            heading_stack.truncate(level.saturating_sub(1));
            heading_stack.push(caps[2].trim().to_string());
            current_start = offset + line.len() + 1;
        } else {
            if current.is_empty() {
                current_start = offset;
            }
            current.push_str(line);
            current.push('\n');
        }
        offset += line.len() + 1;
    }

    if !current.trim().is_empty() || sections.is_empty() {
        sections.push(Section {
            heading_path: heading_stack,
            text: current,
            char_start: current_start,
        });
    }
    sections
}

fn tokenize(input: &str) -> Vec<String> {
    input
        .split(|ch: char| !ch.is_alphanumeric())
        .filter_map(|term| {
            let term = term.trim().to_ascii_lowercase();
            (term.len() > 1).then_some(term)
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect()
}

fn score_chunk(text: &str, query_terms: &[String]) -> f64 {
    let lowered = text.to_ascii_lowercase();
    let mut score = 0.0;
    for term in query_terms {
        let count = lowered.matches(term).count();
        if count > 0 {
            score += 1.0 + (count as f64).ln();
        }
    }
    score / (query_terms.len() as f64)
}

fn to_result(chunk: DocumentChunk, document: LiteratureDocument, score: f64) -> SearchResult {
    SearchResult {
        document_id: chunk.document_id.clone(),
        title: document.metadata.title.clone(),
        chunk_id: chunk.chunk_id.clone(),
        score,
        text: chunk.text,
        citation: ChunkCitation {
            section: (!chunk.heading_path.is_empty()).then(|| chunk.heading_path.join(" > ")),
            ordinal: chunk.ordinal,
            source_path: document.source_path,
            doi: document.metadata.doi,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents::add_document;
    use crate::types::{AddDocumentRequest, PaperMetadata, SearchRequest};
    use tempfile::TempDir;

    #[test]
    fn chunks_by_markdown_heading_and_searches() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("rag.md");
        fs::write(
            &source,
            "# Abstract\nRetrieval augmented generation helps agents.\n# Method\nThe retriever indexes papers.",
        )
        .unwrap();
        let store = LiteratureStore::new(temp.path().join("kb"));
        let added = add_document(
            &store,
            AddDocumentRequest {
                path: source,
                metadata: PaperMetadata {
                    title: Some("RAG Paper".into()),
                    ..PaperMetadata::default()
                },
            },
        )
        .unwrap();

        let built = build_index(
            &store,
            BuildIndexRequest {
                document_id: Some(added.document.document_id),
                chunk_chars: 256,
            },
        )
        .unwrap();
        assert_eq!(built.indexed_documents, 1);
        assert_eq!(built.chunk_count, 2);

        let results = search_index(
            &store,
            SearchRequest {
                query: "retriever papers".into(),
                limit: 3,
            },
        )
        .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].citation.section.as_deref(), Some("Method"));
    }
}
