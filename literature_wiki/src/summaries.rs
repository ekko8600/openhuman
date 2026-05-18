use crate::index::search_index;
use crate::llm::{deepseek_chat, summary_messages, DeepSeekConfig};
use crate::store::LiteratureStore;
use crate::types::{LiteratureSummary, SearchRequest, SummaryRequest};
use anyhow::Result;
use chrono::Utc;

pub fn create_summary(
    store: &LiteratureStore,
    request: SummaryRequest,
    config: &DeepSeekConfig,
) -> Result<LiteratureSummary> {
    let document = store.load_document(&request.document_id)?;
    let query = request
        .focus
        .clone()
        .unwrap_or_else(|| "abstract method contribution limitation experiment conclusion".into());
    let mut results = search_index(
        store,
        SearchRequest {
            query,
            limit: request.max_chunks.max(1),
        },
    )?;
    results.retain(|result| result.document_id == request.document_id);

    if results.is_empty() {
        let chunks = store.load_chunks(&request.document_id)?;
        results = chunks
            .into_iter()
            .take(request.max_chunks.max(1))
            .map(|chunk| crate::types::SearchResult {
                document_id: chunk.document_id.clone(),
                title: document.metadata.title.clone(),
                chunk_id: chunk.chunk_id.clone(),
                score: 0.0,
                text: chunk.text,
                citation: crate::types::ChunkCitation {
                    section: (!chunk.heading_path.is_empty())
                        .then(|| chunk.heading_path.join(" > ")),
                    ordinal: chunk.ordinal,
                    source_path: document.source_path.clone(),
                    doi: document.metadata.doi.clone(),
                },
            })
            .collect();
    }

    let context = results
        .iter()
        .map(|result| {
            format!(
                "[{} | section: {} | ordinal: {}]\n{}",
                result.chunk_id,
                result.citation.section.as_deref().unwrap_or("unknown"),
                result.citation.ordinal,
                result.text
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n---\n\n");

    let messages = summary_messages(
        document.metadata.title.as_deref(),
        &request.language,
        request.focus.as_deref(),
        &context,
    );
    let summary_markdown = deepseek_chat(config, messages)?;
    let cited_chunks = results
        .iter()
        .map(|result| result.chunk_id.clone())
        .collect::<Vec<_>>();

    let summary = LiteratureSummary {
        document_id: document.document_id.clone(),
        title: document.metadata.title.clone(),
        language: request.language,
        model: config.model.clone(),
        focus: request.focus,
        summary_markdown,
        cited_chunks,
        created_at: Utc::now(),
    };
    store.save_summary(&summary)?;
    Ok(summary)
}
