# Literature Core knowledge base

`literature-core` is a new agent-first project inside this repository for building a personal literature knowledge base without the OpenHuman desktop frontend.

## Purpose

The project narrows OpenHuman's core idea—local, schema-described operations that agents can call—into a literature workflow for Codex, Cursor, opencode, and similar coding agents.

It focuses on:

- importing local papers and notes;
- extracting text from Markdown, `.TXT`/`.txt`, PDF, and HTML;
- storing deterministic document metadata;
- chunking documents by Markdown headings and fixed-size windows;
- building a local keyword index;
- returning citation-ready search snippets.

## Project layout

```text
literature-core/
  Cargo.toml
  src/
    cli.rs        # CLI command surface for agents
    documents.rs  # ingest/list/get/remove operations
    index.rs      # chunking and keyword search
    manifest.rs   # method schemas
    parser.rs     # md/txt/pdf/html text extraction
    rpc.rs        # stdin JSON-RPC dispatch
    store.rs      # workspace filesystem store
    types.rs      # serializable API types
skills/
  literature-ingest/SKILL.md
  literature-search/SKILL.md
  literature-summarize/SKILL.md
```

## Workspace

By default, the library is stored in:

```text
~/.literature-core/
  documents/<document_id>/
    original.<ext>
    text.md
    metadata.json
    chunks.json
  index/chunks.json
  collections/
```

For agent runs, prefer an explicit workspace:

```bash
export LITERATURE_CORE_WORKSPACE="$PWD/.literature-core"
```

or pass:

```bash
--workspace "$PWD/.literature-core"
```

## CLI examples

Import a document:

```bash
cargo run --manifest-path literature-core/Cargo.toml -- \
  --workspace .literature-core \
  document add ~/papers/example.pdf --title "Example Paper"
```

Build the index:

```bash
cargo run --manifest-path literature-core/Cargo.toml -- \
  --workspace .literature-core \
  index build
```

Search with citation-ready snippets:

```bash
cargo run --manifest-path literature-core/Cargo.toml -- \
  --workspace .literature-core \
  index search "retrieval augmented generation limitations" --limit 8
```

Export schemas:

```bash
cargo run --manifest-path literature-core/Cargo.toml -- schema
```

## JSON-RPC examples

```bash
printf '%s' '{"jsonrpc":"2.0","id":1,"method":"literature.index.search","params":{"query":"agent memory","limit":5}}' | \
  cargo run --manifest-path literature-core/Cargo.toml -- \
  --workspace .literature-core \
  json-rpc
```

## Current limitations

- PDF support depends on embedded/extractable text. OCR is not implemented.
- Search is deterministic keyword scoring, not vector search.
- Chunk citations include sections and chunk ordinals, but not reliable PDF page numbers yet.
- Summarization is an agent workflow built on search results; the core does not call an LLM directly.
