# PRVW — PR Review Viewer

A desktop diff viewer that fetches GitHub PRs, structures diffs into hunks, and uses LLM (Codex CLI) to group changes by intent — so you can review one purpose at a time.

Built with **Tauri v2 + React + TypeScript + Rust**.

![PRVW Screenshot](docs/screenshot.png)

## Stack

| Tool | Role | Install |
|------|------|---------|
| **Node.js** (v18+) | Frontend build | https://nodejs.org/ |
| **Rust** (stable) | Backend | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **GitHub CLI (`gh`)** | PR list & diff | `brew install gh` |
| **Codex CLI (`codex`)** | Intent analysis (optional) | https://github.com/openai/codex |

## Setup

```bash
npm install
gh auth login
codex login   # optional — only needed for intent analysis
```

## Development

```bash
npm run tauri dev
```

## Usage

1. Enter a repo (e.g. `facebook/react`) and click **Fetch PRs**
2. Select a PR — diff is fetched, parsed, and displayed in the center pane
3. Click **Run** — hunks are grouped by change intent (cached for instant recall)
4. Click a group to filter the diff; summary, rationale, and checklists appear in the right pane
5. Check off groups as you review them
6. Use **Refine** on a group to split it into smaller sub-groups
7. Use **Re-run** to bypass cache and re-analyze

## Features

- **Intent grouping** — LLM groups hunks by change purpose with category labels (schema, logic, api, ui, test, config, docs, refactor)
- **Risk coloring** — Group badges are colored by risk level (green/yellow/red)
- **Cosmetic detection** — Non-substantive changes (formatting, whitespace, lock files) are auto-collapsed
- **Disk cache** — Analysis results and PR diffs are cached to disk for instant recall across sessions
- **Substantive filter** — Toggle to collapse/expand cosmetic hunks
- **File filters** — Filter by extension, hide test files
- **Settings** — Configure Codex model, response language, and manage cache

## Troubleshooting

| Error | Fix |
|-------|-----|
| `gh is not installed` | `brew install gh` |
| `gh is not authenticated` | Run `gh auth login` |
| `Codex CLI is not installed` | Install Codex (diff viewing works without it) |
| `Codex CLI is not authenticated` | Run `codex login` |
| Empty diff | PR has no changes — check branch comparison |
| Analysis validation error | Codex output was malformed — retry usually fixes it |

## Project Structure

```
prvw/
├── src/                        # Frontend (React + TS)
│   ├── App.tsx                 # Main shell — header, routing, state
│   ├── App.css                 # Styles
│   ├── types.ts                # TypeScript type definitions
│   ├── constants.ts            # Shared constants
│   ├── components/
│   │   ├── PrList.tsx          # PR list table
│   │   ├── GroupsPane.tsx      # Left pane — intent groups
│   │   ├── DiffPane.tsx        # Center pane — diff view
│   │   ├── SummaryPane.tsx     # Right pane — summary / AI comments
│   │   └── SettingsModal.tsx   # Settings & cache management
│   ├── hooks/
│   │   ├── useAnalysis.ts      # Codex analysis + refine logic
│   │   ├── useCodexApi.ts      # Tauri invoke wrappers
│   │   ├── useGroupFiltering.ts # Group selection & filtering
│   │   ├── usePrDiff.ts        # PR diff fetching
│   │   ├── usePrList.ts        # PR list fetching
│   │   ├── useRepoHistory.ts   # Repo input history
│   │   └── useSettings.ts     # Settings persistence
│   └── utils/
│       └── classifyFile.ts     # File category classification
├── src-tauri/                  # Backend (Rust)
│   ├── schemas/                # JSON schemas for Codex output
│   └── src/
│       ├── lib.rs              # Tauri app init & command registration
│       ├── types.rs            # Shared structs
│       ├── gh.rs               # GitHub CLI commands
│       ├── diff_parser.rs      # Unified diff parser
│       ├── codex.rs            # Codex CLI commands (analyze, refine)
│       ├── codex_runner.rs     # Codex subprocess execution
│       ├── cache.rs            # Disk cache utilities
│       └── validation.rs       # Analysis result validation
├── index.html
└── package.json
```
