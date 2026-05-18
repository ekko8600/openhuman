# literature-summarize

Use this skill when a user asks for a paper summary, reading note, or comparison grounded in their local literature knowledge base.

## Current implementation status

`literature-core` currently provides ingestion, chunking, schema export, JSON-RPC dispatch, and keyword search. It does not yet call an LLM directly. Agents should summarize by retrieving relevant chunks and then composing the summary themselves with explicit citations.

## Recommended workflow

1. Run `document get <document_id>` to inspect metadata.
2. Run focused `index search` queries for abstract, method, contribution, limitation, experiment, and conclusion terms.
3. Produce a Markdown note with these sections:
   - TL;DR
   - Problem
   - Method
   - Key contributions
   - Evidence / experiments
   - Limitations
   - Questions for follow-up
   - Cited chunks

## Commands

```bash
cargo run --manifest-path literature-core/Cargo.toml -- document get <document_id>
cargo run --manifest-path literature-core/Cargo.toml -- index search "<paper topic> method contribution limitation" --limit 12
```

## Citation rule

Every non-obvious claim should be traceable to a returned chunk. Include `document_id`, `chunk_id`, and section in the note.
