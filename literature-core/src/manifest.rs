use crate::types::ControllerSchema;

pub fn controller_schemas() -> Vec<ControllerSchema> {
    vec![
        ControllerSchema {
            method: "literature.document.add".into(),
            description: "Import a .md, .TXT/.txt, .PDF/.pdf, or .html/.htm document into the local literature knowledge base.".into(),
            required: vec!["path".into()],
            optional: vec!["title".into(), "doi".into(), "year".into(), "authors".into()],
        },
        ControllerSchema {
            method: "literature.document.list".into(),
            description: "List imported literature documents and metadata.".into(),
            required: vec![],
            optional: vec![],
        },
        ControllerSchema {
            method: "literature.document.get".into(),
            description: "Fetch one imported literature document by document_id.".into(),
            required: vec!["document_id".into()],
            optional: vec![],
        },
        ControllerSchema {
            method: "literature.document.remove".into(),
            description: "Remove one imported literature document and its indexed chunks.".into(),
            required: vec!["document_id".into()],
            optional: vec![],
        },
        ControllerSchema {
            method: "literature.index.build".into(),
            description: "Build keyword-search chunks for one document or the full local library.".into(),
            required: vec![],
            optional: vec!["document_id".into(), "chunk_chars".into()],
        },
        ControllerSchema {
            method: "literature.index.search".into(),
            description: "Search indexed literature chunks and return citation-ready snippets.".into(),
            required: vec!["query".into()],
            optional: vec!["limit".into()],
        },
    ]
}
