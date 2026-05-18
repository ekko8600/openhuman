# Agent skills for Literature Core

This repository now includes local skill instructions for using `literature-core` from Codex, Cursor, opencode, and similar agents.

## Skills

| Skill | Use case |
| --- | --- |
| `skills/literature-ingest/SKILL.md` | Import `.md`, `.TXT`, `.pdf`, and `.html` documents into a local workspace. |
| `skills/literature-search/SKILL.md` | Search indexed chunks and answer with local citations. |
| `skills/literature-summarize/SKILL.md` | Compose cited paper summaries and reading notes using retrieved chunks. |

## Agent contract

Agents should treat `literature-core` as a local tool boundary:

1. Prefer CLI calls for simple workflows.
2. Use stdin JSON-RPC when a structured method call is easier to compose.
3. Keep the workspace explicit for reproducibility.
4. Preserve returned citation fields in user-facing answers.
5. Do not claim OCR, vector search, or direct LLM summarization support until those features are implemented.

## Recommended command prefix

```bash
cargo run --manifest-path literature-core/Cargo.toml -- --workspace .literature-core
```

## Available methods

Run this command for the current machine-readable schema:

```bash
cargo run --manifest-path literature-core/Cargo.toml -- schema
```

Initial methods:

- `literature.document.add`
- `literature.document.list`
- `literature.document.get`
- `literature.document.remove`
- `literature.index.build`
- `literature.index.search`
- `literature.schema`
