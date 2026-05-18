use crate::documents::{add_document, get_document, list_documents, remove_document};
use crate::index::{build_index, search_index};
use crate::manifest::controller_schemas;
use crate::store::LiteratureStore;
use crate::types::{AddDocumentRequest, BuildIndexRequest, PaperMetadata, SearchRequest};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: Option<String>,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: &'static str,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
}

pub fn handle_json_rpc(store: &LiteratureStore, request: JsonRpcRequest) -> JsonRpcResponse {
    let id = request.id.clone();
    match dispatch(store, request) {
        Ok(result) => JsonRpcResponse {
            jsonrpc: "2.0",
            id,
            result: Some(result),
            error: None,
        },
        Err(error) => JsonRpcResponse {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32000,
                message: error.to_string(),
            }),
        },
    }
}

pub fn dispatch(store: &LiteratureStore, request: JsonRpcRequest) -> Result<Value> {
    if request
        .jsonrpc
        .as_deref()
        .is_some_and(|version| version != "2.0")
    {
        bail!("unsupported JSON-RPC version");
    }
    match request.method.as_str() {
        "schema" | "literature.schema" => Ok(serde_json::to_value(controller_schemas())?),
        "literature.document.add" => {
            let params = object_params(request.params)?;
            let path = required_string(&params, "path")?;
            let metadata = metadata_from_params(&params)?;
            Ok(serde_json::to_value(add_document(
                store,
                AddDocumentRequest {
                    path: PathBuf::from(path),
                    metadata,
                },
            )?)?)
        }
        "literature.document.list" => Ok(serde_json::to_value(list_documents(store)?)?),
        "literature.document.get" => {
            let params = object_params(request.params)?;
            Ok(serde_json::to_value(get_document(
                store,
                &required_string(&params, "document_id")?,
            )?)?)
        }
        "literature.document.remove" => {
            let params = object_params(request.params)?;
            Ok(json!({
                "removed": remove_document(store, &required_string(&params, "document_id")?)?
            }))
        }
        "literature.index.build" => {
            let request = serde_json::from_value::<BuildIndexRequest>(request.params)
                .context("invalid build-index params")?;
            Ok(serde_json::to_value(build_index(store, request)?)?)
        }
        "literature.index.search" => {
            let request = serde_json::from_value::<SearchRequest>(request.params)
                .context("invalid search params")?;
            Ok(serde_json::to_value(search_index(store, request)?)?)
        }
        method => bail!("unknown method: {method}"),
    }
}

fn object_params(params: Value) -> Result<serde_json::Map<String, Value>> {
    match params {
        Value::Object(map) => Ok(map),
        Value::Null => Ok(Default::default()),
        _ => bail!("params must be a JSON object"),
    }
}

fn required_string(params: &serde_json::Map<String, Value>, key: &str) -> Result<String> {
    params
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .with_context(|| format!("missing required string param: {key}"))
}

fn metadata_from_params(params: &serde_json::Map<String, Value>) -> Result<PaperMetadata> {
    let authors = params
        .get("authors")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Ok(PaperMetadata {
        title: params
            .get("title")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        authors,
        year: params
            .get("year")
            .and_then(Value::as_i64)
            .map(|year| year as i32),
        doi: params
            .get("doi")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        arxiv_id: params
            .get("arxiv_id")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        source_url: params
            .get("source_url")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn dispatches_add_build_and_search() {
        let temp = TempDir::new().unwrap();
        let paper = temp.path().join("paper.md");
        fs::write(&paper, "# Intro\nAgentic literature search").unwrap();
        let store = LiteratureStore::new(temp.path().join("kb"));

        let added = dispatch(
            &store,
            JsonRpcRequest {
                jsonrpc: Some("2.0".into()),
                id: None,
                method: "literature.document.add".into(),
                params: json!({"path": paper, "title": "Agent Paper"}),
            },
        )
        .unwrap();
        let document_id = added["document"]["document_id"]
            .as_str()
            .unwrap()
            .to_string();

        dispatch(
            &store,
            JsonRpcRequest {
                jsonrpc: Some("2.0".into()),
                id: None,
                method: "literature.index.build".into(),
                params: json!({"document_id": document_id}),
            },
        )
        .unwrap();

        let results = dispatch(
            &store,
            JsonRpcRequest {
                jsonrpc: Some("2.0".into()),
                id: None,
                method: "literature.index.search".into(),
                params: json!({"query": "literature", "limit": 2}),
            },
        )
        .unwrap();
        assert_eq!(results.as_array().unwrap().len(), 1);
    }
}
