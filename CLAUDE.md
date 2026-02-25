# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

PRVW is a Tauri v2 desktop app for PR review. It fetches GitHub PR diffs via `gh`, parses them into hunks, and uses Codex CLI to group hunks by change intent for structured review.

## Build & Dev Commands

```bash
npm run tauri dev       # Full dev mode (Vite frontend + Rust backend, hot reload)
npm run tauri build     # Production build

# Frontend only
npm run dev             # Vite dev server on port 1420
npm run build           # TypeScript check + Vite bundle to dist/

# Rust only
cargo check --manifest-path src-tauri/Cargo.toml   # Type check
cargo build --manifest-path src-tauri/Cargo.toml    # Build

# Type checking
npx tsc --noEmit        # TypeScript strict mode check
```

First `tauri dev` takes several minutes for Rust compilation; subsequent runs use incremental builds.

## Architecture

**Tauri IPC bridge**: React frontend calls Rust backend via `invoke()`. Rust commands execute `gh` and `codex` as subprocesses (`std::process::Command` with argument arrays, never shell strings).

### Rust Backend (`src-tauri/src/commands.rs`)

5 Tauri commands registered in `lib.rs`:

| Command | Purpose | External tool |
|---------|---------|---------------|
| `list_prs` | Fetch PR list via `gh pr list --json` | gh |
| `get_pr_diff` | Fetch raw diff via `gh pr diff --patch` | gh |
| `parse_diff` | Regex-based unified diff parser → `Vec<Hunk>` | none |
| `split_large_hunks` | Split hunks >100 lines via Codex | codex |
| `analyze_intents_with_codex` | Group hunks by intent via Codex | codex |

All `gh` commands run with `GH_PAGER=cat`, `NO_COLOR=1`, `GH_FORCE_TTY=0`. Codex runs in temp dirs with `--sandbox read-only`, `--full-auto`, `--output-schema`.

### React Frontend (`src/App.tsx`)

Single-file app with useState/useMemo. 3-pane layout:
- **Left**: Intent groups, Codex model/lang inputs, run button
- **Center**: Filtered diff view with file category toggles (generated/test/docs/config)
- **Right**: Analysis summary, rationale, checklists

### Data Flow

```
fetchPrs → PR table → selectPr → get_pr_diff → parse_diff → hunks displayed
  → runAnalysis → [split_large_hunks if >100 lines] → analyze_intents_with_codex → groups displayed
```

## Key Conventions

- **Serde mapping**: Rust `snake_case` ↔ TypeScript `camelCase` via `#[serde(rename_all = "camelCase")]`
- **Hunk IDs**: Sequential `H1`, `H2`, ...; split sub-hunks use `H5.1`, `H5.2`
- **Analysis validation**: All hunk IDs must exist, no duplicates, 100% coverage (groups + unassigned)
- **Codex integration**: Schema-constrained output (`--output-schema`); prompt includes ordering instruction (data model → logic → API → UI → tests → config)
- **localStorage persistence**: `prvw:repo`, `prvw:codexModel`, `prvw:lang`
- **Error pattern**: Rust returns `Result<T, String>`; frontend shows in error bar with dismiss button

## Runtime Dependencies

- **gh** (GitHub CLI): Required. Must be authenticated (`gh auth login`)
- **codex** (Codex CLI): Optional. Needed for intent analysis/hunk splitting. Config at `~/.codex/config.toml`
- App works without Codex for basic diff viewing
