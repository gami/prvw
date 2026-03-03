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

### Rust Backend (`src-tauri/src/`)

7 Tauri commands registered in `lib.rs`:

| Command | File | Purpose | External tool |
|---------|------|---------|---------------|
| `list_prs` | `gh.rs` | Fetch PR list via `gh pr list --json` | gh |
| `get_pr_diff` | `gh.rs` | Fetch raw diff via `gh pr diff --patch` (cached to disk) | gh |
| `parse_diff` | `diff_parser.rs` | Regex-based unified diff parser → `Vec<Hunk>` | none |
| `analyze_intents_with_codex` | `codex.rs` | Group hunks by intent via Codex (cached to disk) | codex |
| `refine_group` | `codex.rs` | Split a group into sub-groups via Codex (cached to disk) | codex |
| `get_cache_size` | `cache.rs` | Return human-readable disk cache size | none |
| `clear_cache` | `cache.rs` | Delete all cached data | none |

Other modules:
- `codex_runner.rs` — Shared Codex subprocess execution (temp dir setup, arg building, output parsing)
- `validation.rs` — Analysis result validation (hunk ID coverage, dedup)
- `cache.rs` — Disk cache utilities (hash key, read/write JSON, size calculation)

All `gh` commands run with `GH_PAGER=cat`, `NO_COLOR=1`, `GH_FORCE_TTY=0`. Codex runs in temp dirs with `--sandbox read-only`, `--full-auto`, `--output-schema`.

### React Frontend (`src/`)

Component-based architecture with custom hooks. 3-pane layout:
- **Left** (`GroupsPane.tsx`): Intent groups with category badges, risk-colored borders, review checkboxes, refine buttons
- **Center** (`DiffPane.tsx`): Filtered diff view with file-level collapse, substantive/cosmetic toggle, extension filters
- **Right** (`SummaryPane.tsx`): PR title, overall summary, group rationale, reviewer checklists, Codex log

Other components:
- `PrList.tsx` — PR selection table (shown before PR is selected)
- `SettingsModal.tsx` — Codex model/language config, cache management

Hooks:
- `useAnalysis.ts` — Codex analysis + refine state management
- `useCodexApi.ts` — Tauri invoke wrappers for Codex commands
- `useGroupFiltering.ts` — Group selection, displayed hunks filtering, reviewed state
- `usePrDiff.ts` — PR diff fetching and parsing
- `usePrList.ts` — PR list fetching
- `useRepoHistory.ts` — Repo input with localStorage history
- `useSettings.ts` — Codex model/lang settings persistence

### Data Flow

```
fetchPrs → PR table → selectPr → get_pr_diff → parse_diff → hunks displayed
  → runAnalysis → analyze_intents_with_codex → groups displayed (first auto-selected)
  → refineGroup → refine_group → sub-groups replace parent group
```

### Disk Cache

Cached in `AppHandle::path().app_data_dir()/cache/` (`~/Library/Application Support/com.masakitakegami.prvw/cache/`):
- `cache/diff/` — PR diffs keyed by `{repo}_{prNumber}.json`
- `cache/analysis/` — Analysis results keyed by hash of (hunks + PR body + model + lang)
- `cache/refine/` — Refine results keyed by hash of (group hunks + group ID + model + lang)

Cache is best-effort (write failures ignored, read failures = cache miss). "Re-run" button bypasses cache via `force` parameter.

## Key Conventions

- **Serde mapping**: Rust `snake_case` ↔ TypeScript `camelCase` via `#[serde(rename_all = "camelCase")]`
- **Hunk IDs**: Sequential `H1`, `H2`, ...
- **Group categories**: `schema`, `logic`, `api`, `ui`, `test`, `config`, `docs`, `refactor`, `other`
- **Analysis validation**: All hunk IDs must exist, no duplicates, 100% coverage (groups + unassigned)
- **Codex integration**: Schema-constrained output (`--output-schema`); prompt includes PR body, hunk count, ordering instruction (data model → logic → API → UI → tests → config)
- **localStorage persistence**: `prvw:repo`, `prvw:codexModel`, `prvw:lang`
- **Error pattern**: Rust returns `Result<T, String>`; frontend shows in error bar with dismiss button

## Runtime Dependencies

- **gh** (GitHub CLI): Required. Must be authenticated (`gh auth login`)
- **codex** (Codex CLI): Optional. Needed for intent analysis/refine. Config at `~/.codex/config.toml`
- App works without Codex for basic diff viewing
