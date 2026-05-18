# literature-ingest

Use this skill when a user asks an agent to build or update a local personal literature knowledge base from papers, notes, or exported web pages.

## Supported inputs

- Markdown: `.md`, `.markdown`
- Plain text: `.txt` and uppercase `.TXT`
- PDF: `.pdf` with extractable text
- HTML: `.html`, `.htm`

## Commands

```bash
cargo run --manifest-path literature-core/Cargo.toml -- document add /path/to/paper.pdf --title "Paper title" --doi "10.xxxx/example"
cargo run --manifest-path literature-core/Cargo.toml -- index build --document-id <document_id>
```

Use `--workspace /path/to/kb` or `LITERATURE_CORE_WORKSPACE=/path/to/kb` to keep the library in a project-local directory.

## JSON-RPC

```json
{"jsonrpc":"2.0","id":1,"method":"literature.document.add","params":{"path":"/path/to/paper.md","title":"Paper title"}}
```

Pipe the request into:

```bash
cargo run --manifest-path literature-core/Cargo.toml -- json-rpc
```

## Agent behavior

1. Import the source document.
2. Build the index for the returned `document_id`.
3. Report the `document_id`, title, detected format, duplicate status, and workspace path.
4. If PDF extraction fails, tell the user OCR is not implemented yet and request a text/Markdown/HTML version.
