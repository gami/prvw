import { useState, useMemo, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { PrListItem, Hunk, ParsedDiff, IntentGroup } from "./types";
import { UNASSIGNED_GROUP_ID } from "./constants";
import { useAnalysis } from "./hooks/useAnalysis";
import { PrList } from "./components/PrList";
import { GroupsPane } from "./components/GroupsPane";
import { DiffPane } from "./components/DiffPane";
import { SummaryPane } from "./components/SummaryPane";
import "./App.css";

const REPO_HISTORY_KEY = "prvw:repoHistory";
const REPO_HISTORY_MAX = 20;

function loadRepoHistory(): string[] {
  try {
    const raw = localStorage.getItem(REPO_HISTORY_KEY);
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

function saveRepoToHistory(repo: string) {
  const history = loadRepoHistory().filter((r) => r !== repo);
  history.unshift(repo);
  localStorage.setItem(REPO_HISTORY_KEY, JSON.stringify(history.slice(0, REPO_HISTORY_MAX)));
}

function App() {
  // ── State ──
  const [repo, setRepo] = useState(() => localStorage.getItem("prvw:repo") ?? "");
  const [repoHistory, setRepoHistory] = useState(loadRepoHistory);
  const [search, setSearch] = useState("");
  const [prs, setPrs] = useState<PrListItem[]>([]);
  const [selectedPr, setSelectedPr] = useState<PrListItem | null>(null);
  const [hunks, setHunks] = useState<Hunk[]>([]);
  const [selectedGroupId, setSelectedGroupId] = useState<string | null>(null);
  const [reviewedGroups, setReviewedGroups] = useState<Set<string>>(new Set());
  const [codexModel, setCodexModel] = useState(() => localStorage.getItem("prvw:codexModel") ?? "");
  const [lang, setLang] = useState(() => localStorage.getItem("prvw:lang") ?? "ja");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState<string | null>(null);

  const { analysis, codexLog, runAnalysis, refineGroup, resetAnalysis } = useAnalysis({
    hunks,
    codexModel,
    lang,
    setError,
    setLoading,
  });

  // ── Auto-fetch on startup if repo is saved ──
  useEffect(() => {
    if (repo.trim()) {
      fetchPrs();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // ── Derived ──
  const selectedGroup: IntentGroup | null = useMemo(() => {
    if (!analysis || !selectedGroupId) return null;
    return analysis.groups.find((g) => g.id === selectedGroupId) ?? null;
  }, [analysis, selectedGroupId]);

  const displayedHunks: Hunk[] = useMemo(() => {
    if (selectedGroupId === UNASSIGNED_GROUP_ID && analysis) {
      return hunks.filter((h) => analysis.unassignedHunkIds.includes(h.id));
    }
    if (selectedGroup) {
      const ids = new Set(selectedGroup.hunkIds);
      return hunks.filter((h) => ids.has(h.id));
    }
    return hunks;
  }, [hunks, selectedGroup, selectedGroupId, analysis]);

  // ── Handlers ──
  async function fetchPrs() {
    setError(null);
    if (!repo.trim()) {
      setError("Please enter a repository (owner/repo).");
      return;
    }
    setLoading("Fetching PRs...");
    try {
      const items = await invoke<PrListItem[]>("list_prs", {
        repo: repo.trim(),
        limit: 30,
        state: "open",
        search: search.trim() || null,
      });
      setPrs(items);
      localStorage.setItem("prvw:repo", repo.trim());
      saveRepoToHistory(repo.trim());
      setRepoHistory(loadRepoHistory());
      setSelectedPr(null);
      setHunks([]);
      resetAnalysis();
      if (items.length === 0) {
        setError("No open PRs found.");
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(null);
    }
  }

  async function selectPr(pr: PrListItem) {
    setError(null);
    setSelectedPr(pr);
    resetAnalysis();
    setSelectedGroupId(null);
    setReviewedGroups(new Set());
    setLoading("Fetching diff...");
    try {
      const diff = await invoke<string>("get_pr_diff", {
        repo: repo.trim(),
        prNumber: pr.number,
      });
      const parsed = await invoke<ParsedDiff>("parse_diff", {
        diffText: diff,
      });
      setHunks(parsed.hunks);
    } catch (e) {
      setError(String(e));
      setHunks([]);
    } finally {
      setLoading(null);
    }
  }

  function toggleReviewed(groupId: string) {
    setReviewedGroups((prev) => {
      const next = new Set(prev);
      if (next.has(groupId)) next.delete(groupId);
      else next.add(groupId);
      return next;
    });
  }

  function goBackToList() {
    setSelectedPr(null);
    setHunks([]);
    resetAnalysis();
    setSelectedGroupId(null);
  }

  // ── Render ──
  return (
    <div className="app">
      {/* Header */}
      <header className="header">
        <div className="header-left">
          <strong className="logo">PRVW</strong>
          <input
            className="input repo-input"
            placeholder="owner/repo"
            list="repo-history"
            value={repo}
            onChange={(e) => setRepo(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && fetchPrs()}
          />
          <datalist id="repo-history">
            {repoHistory.map((r) => (
              <option key={r} value={r} />
            ))}
          </datalist>
          <input
            className="input search-input"
            placeholder="Search PRs (optional)"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && fetchPrs()}
          />
          <button className="btn btn-primary" onClick={fetchPrs} disabled={!!loading}>
            Fetch PRs
          </button>
        </div>
        <div className="header-right">
          {selectedPr && (
            <>
              <span className="selected-pr">
                #{selectedPr.number} {selectedPr.title}
              </span>
              <button
                className="btn btn-ghost"
                disabled={!selectedPr.url}
                onClick={() => {
                  if (selectedPr.url) {
                    openUrl(selectedPr.url).catch((e) => setError(String(e)));
                  }
                }}
              >
                Open PR
              </button>
            </>
          )}
        </div>
      </header>

      {/* Error / Loading */}
      {error && (
        <div className="error-bar">
          <span>{error}</span>
          <button className="btn-close" onClick={() => setError(null)}>×</button>
        </div>
      )}
      {loading && <div className="loading-bar">{loading}</div>}

      {/* PR List (shown when no PR selected) */}
      {!selectedPr && prs.length > 0 && (
        <PrList prs={prs} onSelect={selectPr} />
      )}

      {/* 3-Pane Layout */}
      {selectedPr && (
        <div className="panes">
          <GroupsPane
            hunks={hunks}
            analysis={analysis}
            selectedGroupId={selectedGroupId}
            reviewedGroups={reviewedGroups}
            codexModel={codexModel}
            lang={lang}
            loading={!!loading}
            onSelectGroup={setSelectedGroupId}
            onToggleReviewed={toggleReviewed}
            onSetCodexModel={setCodexModel}
            onSetLang={setLang}
            onRunAnalysis={runAnalysis}
            onRefineGroup={refineGroup}
            onBack={goBackToList}
          />
          <DiffPane
            hunks={displayedHunks}
            selectedGroup={selectedGroup}
            selectedGroupId={selectedGroupId}
          />
          <SummaryPane
            analysis={analysis}
            selectedGroup={selectedGroup}
            codexLog={codexLog}
          />
        </div>
      )}
    </div>
  );
}

export default App;
