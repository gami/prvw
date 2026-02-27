import { useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useRepoHistory } from "./hooks/useRepoHistory";
import { usePrList } from "./hooks/usePrList";
import { usePrDiff } from "./hooks/usePrDiff";
import { useAnalysis } from "./hooks/useAnalysis";
import { useGroupFiltering } from "./hooks/useGroupFiltering";
import { PrList } from "./components/PrList";
import { GroupsPane } from "./components/GroupsPane";
import { DiffPane } from "./components/DiffPane";
import { SummaryPane } from "./components/SummaryPane";
import "./App.css";

function App() {
  // ── Global UI state ──
  const [search, setSearch] = useState("");
  const [codexModel, setCodexModel] = useState(() => localStorage.getItem("prvw:codexModel") ?? "");
  const [lang, setLang] = useState(() => localStorage.getItem("prvw:lang") ?? "ja");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState<string | null>(null);

  // ── Hooks ──
  const { repo, setRepo, repoHistory, persistRepo } = useRepoHistory();

  const { prs, fetchPrs } = usePrList({
    repo,
    search,
    onFetched: (r) => {
      persistRepo(r);
      clearSelection();
      resetAnalysis();
    },
    setError,
    setLoading,
  });

  const { selectedPr, hunks, selectPr: rawSelectPr, clearSelection } = usePrDiff({
    repo,
    setError,
    setLoading,
  });

  const { analysis, codexLog, runAnalysis, refineGroup, resetAnalysis } = useAnalysis({
    hunks,
    codexModel,
    lang,
    setError,
    setLoading,
  });

  const {
    selectedGroupId,
    setSelectedGroupId,
    selectedGroup,
    displayedHunks,
    reviewedGroups,
    toggleReviewed,
    resetFiltering,
  } = useGroupFiltering(hunks, analysis);

  // ── Handlers ──
  function handleSelectPr(pr: typeof prs[number]) {
    resetAnalysis();
    resetFiltering();
    rawSelectPr(pr);
  }

  function goBackToList() {
    clearSelection();
    resetAnalysis();
    resetFiltering();
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
        <PrList prs={prs} onSelect={handleSelectPr} />
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
