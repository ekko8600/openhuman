# Literature Wiki

`literature_wiki` is a standalone agent-first project for building a personal literature knowledge base. It intentionally lives under `literature_wiki/` so the original OpenHuman desktop app, frontend, and unrelated community-data-source code do not need to be built or modified.

## Purpose

The project narrows the useful OpenHuman idea—local, schema-described operations that agents can call—into a literature workflow for Codex, Cursor, opencode, and similar coding agents.

It focuses on:

- importing local papers and notes;
- extracting text from Markdown, `.TXT`/`.txt`, PDF, and HTML;
- storing deterministic document metadata;
- chunking documents by Markdown headings and fixed-size windows;
- building a local keyword index;
- creating DeepSeek V4 Pro-backed Markdown analyses/summaries;
- returning citation-ready search snippets for downstream agent answers.

## Project layout

```text
literature_wiki/
  Cargo.toml
  src/
    cli.rs        # CLI command surface for agents
    documents.rs  # ingest/list/get/remove operations
    index.rs      # chunking and keyword search
    llm.rs        # DeepSeek V4 Pro chat-completions client
    manifest.rs   # method schemas
    parser.rs     # md/txt/pdf/html text extraction
    rpc.rs        # stdin JSON-RPC dispatch
    store.rs      # workspace filesystem store
    summaries.rs  # retrieval-grounded analysis/summarization
    types.rs      # serializable API types
  docs/
  skills/
```

## Workspace

By default, the library is stored in:

```text
~/.literature-wiki/
  documents/<document_id>/
    original.<ext>
    text.md
    metadata.json
    chunks.json
  index/chunks.json
  summaries/<document_id>.json
  collections/
```

For agent runs, prefer an explicit workspace:

```bash
export LITERATURE_WIKI_WORKSPACE="$PWD/.literature-wiki"
```

or pass:

```bash
--workspace "$PWD/.literature-wiki"
```

## DeepSeek V4 Pro configuration

Summaries and analyses call the DeepSeek chat-completions API. Configure it with:

```bash
export DEEPSEEK_API_KEY="..."
export DEEPSEEK_MODEL="deepseek-v4-pro"          # default
export DEEPSEEK_API_BASE="https://api.deepseek.com" # default
```

The core does not send files directly to the model. It first indexes local text chunks, selects relevant chunks, then sends those chunk snippets with chunk IDs in the prompt so the generated Markdown can cite local evidence.

## CLI examples

Import a document:

```bash
cargo run --manifest-path literature_wiki/Cargo.toml -- \
  --workspace .literature-wiki \
  document add ~/papers/example.pdf --title "Example Paper"
```

Build the index:

```bash
cargo run --manifest-path literature_wiki/Cargo.toml -- \
  --workspace .literature-wiki \
  index build
```

Search with citation-ready snippets:

```bash
cargo run --manifest-path literature_wiki/Cargo.toml -- \
  --workspace .literature-wiki \
  index search "retrieval augmented generation limitations" --limit 8
```

Create a DeepSeek-backed analysis:

```bash
cargo run --manifest-path literature_wiki/Cargo.toml -- \
  --workspace .literature-wiki \
  summary create <document_id> --language Chinese --focus "method and limitations" --max-chunks 12
```

Export schemas:

```bash
cargo run --manifest-path literature_wiki/Cargo.toml -- schema
```

## JSON-RPC examples

```bash
printf '%s' '{"jsonrpc":"2.0","id":1,"method":"literature.summary.create","params":{"document_id":"doc_x","language":"Chinese","focus":"contributions and limitations","max_chunks":12}}' | \
  cargo run --manifest-path literature_wiki/Cargo.toml -- \
  --workspace .literature-wiki \
  json-rpc
```

## Current limitations

- PDF support depends on embedded/extractable text. OCR is not implemented.
- Search is deterministic keyword scoring, not vector search.
- Chunk citations include sections and chunk ordinals, but not reliable PDF page numbers yet.
