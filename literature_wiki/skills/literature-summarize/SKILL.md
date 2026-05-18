# literature-summarize

Use this skill when a user asks for a paper summary, reading note, or comparison grounded in their local literature wiki.

## Model backend

Summaries call the DeepSeek V4 Pro-compatible chat-completions API. Before running this skill, ensure:

```bash
export DEEPSEEK_API_KEY="..."
export DEEPSEEK_MODEL="deepseek-v4-pro"
```

`DEEPSEEK_API_BASE` defaults to `https://api.deepseek.com`.

## Recommended workflow

1. Run `document get <document_id>` to inspect metadata.
2. Ensure `index build --document-id <document_id>` has been run.
3. Run `summary create` with the target language and focus.
4. Return the generated Markdown and preserve cited chunk IDs.

## Commands

```bash
cargo run --manifest-path literature_wiki/Cargo.toml -- --workspace .literature-wiki document get <document_id>
cargo run --manifest-path literature_wiki/Cargo.toml -- --workspace .literature-wiki summary create <document_id> --language Chinese --focus "method, contributions, limitations" --max-chunks 12
```

## JSON-RPC

```json
{"jsonrpc":"2.0","id":1,"method":"literature.summary.create","params":{"document_id":"doc_x","language":"Chinese","focus":"method and limitations","max_chunks":12}}
```

## Citation rule

Every non-obvious claim should be traceable to a returned or generated chunk reference. Include `document_id`, `chunk_id`, and section in the final note.
