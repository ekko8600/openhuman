# literature-search

Use this skill when the answer should come from the user's local literature knowledge base rather than general web knowledge.

## Commands

```bash
cargo run --manifest-path literature-core/Cargo.toml -- index search "retrieval augmented generation limitations" --limit 8
```

## JSON-RPC

```json
{"jsonrpc":"2.0","id":1,"method":"literature.index.search","params":{"query":"agentic retrieval","limit":8}}
```

## Output expectations

Always preserve citation fields returned by `literature-core`:

- `document_id`
- `title`
- `chunk_id`
- `citation.section`
- `citation.ordinal`
- `citation.source_path`
- `citation.doi`

When answering the user, cite the local document title or path and include the section if available. Do not invent page numbers; this first version tracks chunk ordinals and sections.

## Failure handling

- If no results are returned, suggest `document list` and `index build`.
- If results are weak, broaden the query and run another search.
- If the knowledge base is empty, switch to `literature-ingest` first.
