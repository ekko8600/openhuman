# Agent skills for Literature Wiki

This standalone project includes local skill instructions for using `literature_wiki` from Codex, Cursor, opencode, and similar agents.

## Skills

| Skill | Use case |
| --- | --- |
| `literature_wiki/skills/literature-ingest/SKILL.md` | Import `.md`, `.TXT`, `.pdf`, and `.html` documents into a local workspace. |
| `literature_wiki/skills/literature-search/SKILL.md` | Search indexed chunks and answer with local citations. |
| `literature_wiki/skills/literature-summarize/SKILL.md` | Call DeepSeek V4 Pro to create cited paper summaries and reading notes from retrieved chunks. |

## Agent contract

Agents should treat `literature_wiki` as a local tool boundary:

1. Prefer CLI calls for simple workflows.
2. Use stdin JSON-RPC when a structured method call is easier to compose.
3. Keep the workspace explicit for reproducibility.
4. Preserve returned citation fields in user-facing answers.
5. Set `DEEPSEEK_API_KEY` before invoking `summary create` or `literature.summary.create`.
6. Do not claim OCR, vector search, or page-accurate PDF citations until those features are implemented.

## Recommended command prefix

```bash
cargo run --manifest-path literature_wiki/Cargo.toml -- --workspace .literature-wiki
```

## Available methods

Run this command for the current machine-readable schema:

```bash
cargo run --manifest-path literature_wiki/Cargo.toml -- schema
```

Initial methods:

- `literature.document.add`
- `literature.document.list`
- `literature.document.get`
- `literature.document.remove`
- `literature.index.build`
- `literature.index.search`
- `literature.summary.create`
- `literature.schema`
