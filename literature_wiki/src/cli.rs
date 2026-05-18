use crate::documents::{add_document, get_document, list_documents, remove_document};
use crate::index::{build_index, search_index};
use crate::llm::DeepSeekConfig;
use crate::manifest::controller_schemas;
use crate::rpc::{handle_json_rpc, JsonRpcRequest};
use crate::store::LiteratureStore;
use crate::summaries::create_summary;
use crate::types::{
    AddDocumentRequest, BuildIndexRequest, PaperMetadata, SearchRequest, SummaryRequest,
};
use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use serde::Serialize;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "literature-wiki")]
#[command(about = "Agent-first personal literature knowledge base CLI")]
pub struct Cli {
    #[arg(long, global = true, env = "LITERATURE_WIKI_WORKSPACE")]
    pub workspace: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(subcommand)]
    Document(DocumentCommand),
    #[command(subcommand)]
    Index(IndexCommand),
    #[command(subcommand)]
    Summary(SummaryCommand),
    Schema,
    /// Read a JSON-RPC request from stdin and write a JSON-RPC response.
    JsonRpc,
}

#[derive(Debug, Subcommand)]
pub enum DocumentCommand {
    Add(AddArgs),
    List,
    Get { document_id: String },
    Remove { document_id: String },
}

#[derive(Debug, Args)]
pub struct AddArgs {
    pub path: PathBuf,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub author: Vec<String>,
    #[arg(long)]
    pub year: Option<i32>,
    #[arg(long)]
    pub doi: Option<String>,
    #[arg(long)]
    pub arxiv_id: Option<String>,
    #[arg(long)]
    pub source_url: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum SummaryCommand {
    Create {
        document_id: String,
        #[arg(long, default_value = "Chinese")]
        language: String,
        #[arg(long)]
        focus: Option<String>,
        #[arg(long, default_value_t = crate::types::default_summary_chunks())]
        max_chunks: usize,
    },
}

#[derive(Debug, Subcommand)]
pub enum IndexCommand {
    Build {
        #[arg(long)]
        document_id: Option<String>,
        #[arg(long, default_value_t = crate::types::default_chunk_chars())]
        chunk_chars: usize,
    },
    Search {
        query: String,
        #[arg(long, default_value_t = crate::types::default_search_limit())]
        limit: usize,
    },
}

pub fn run(cli: Cli) -> Result<()> {
    let store = match cli.workspace {
        Some(path) => LiteratureStore::new(path),
        None => LiteratureStore::from_env_or_default()?,
    };

    match cli.command {
        Command::Document(DocumentCommand::Add(args)) => print_json(&add_document(
            &store,
            AddDocumentRequest {
                path: args.path,
                metadata: PaperMetadata {
                    title: args.title,
                    authors: args.author,
                    year: args.year,
                    doi: args.doi,
                    arxiv_id: args.arxiv_id,
                    source_url: args.source_url,
                },
            },
        )?),
        Command::Document(DocumentCommand::List) => print_json(&list_documents(&store)?),
        Command::Document(DocumentCommand::Get { document_id }) => {
            print_json(&get_document(&store, &document_id)?)
        }
        Command::Document(DocumentCommand::Remove { document_id }) => {
            print_json(&serde_json::json!({"removed": remove_document(&store, &document_id)?}))
        }
        Command::Index(IndexCommand::Build {
            document_id,
            chunk_chars,
        }) => print_json(&build_index(
            &store,
            BuildIndexRequest {
                document_id,
                chunk_chars,
            },
        )?),
        Command::Index(IndexCommand::Search { query, limit }) => {
            print_json(&search_index(&store, SearchRequest { query, limit })?)
        }
        Command::Summary(SummaryCommand::Create {
            document_id,
            language,
            focus,
            max_chunks,
        }) => {
            let config = DeepSeekConfig::from_env()?;
            print_json(&create_summary(
                &store,
                SummaryRequest {
                    document_id,
                    language,
                    focus,
                    max_chunks,
                },
                &config,
            )?)
        }
        Command::Schema => print_json(&controller_schemas()),
        Command::JsonRpc => {
            let mut body = String::new();
            io::stdin().read_to_string(&mut body)?;
            let request: JsonRpcRequest =
                serde_json::from_str(&body).context("invalid JSON-RPC request")?;
            print_json(&handle_json_rpc(&store, request))
        }
    }
}

fn print_json<T: Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
