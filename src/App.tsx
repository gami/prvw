import { useState, useEffect, useRef, useMemo } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useRepoHistory } from "./hooks/useRepoHistory";
import { usePrList } from "./hooks/usePrList";
import { usePrDiff } from "./hooks/usePrDiff";
import { useAnalysis } from "./hooks/useAnalysis";
import { useSettings } from "./hooks/useSettings";
import { useGroupFiltering } from "./hooks/useGroupFiltering";
import { PrList } from "./components/PrList";
import { GroupsPane } from "./components/GroupsPane";
import { DiffPane } from "./components/DiffPane";
import { SummaryPane } from "./components/SummaryPane";
import { SettingsModal } from "./components/SettingsModal";
import "./App.css";

function App() {
  // ── Global UI state ──
  const [search, setSearch] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState<string | null>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);

  // ── Settings ──
  const { codexModel, lang, hasSettings, saveSettings } = useSettings();
  const autoRunTriggered = useRef(false);

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

  const { analysis, codexLog, fromCache, runAnalysis, refineGroup, resetAnalysis } = useAnalysis({
    hunks,
    prBody: selectedPr?.body ?? null,
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

  const nonSubstantiveHunkIds = useMemo(
    () => new Set(analysis?.nonSubstantiveHunkIds ?? []),
    [analysis],
  );

  // ── Auto-run analysis when hunks are loaded ──
  useEffect(() => {
    if (
      hunks.length > 0 &&
      !analysis &&
      !loading &&
      hasSettings &&
      !autoRunTriggered.current
    ) {
      autoRunTriggered.current = true;
      runAnalysis();
    }
  }, [hunks, analysis, loading, hasSettings]);

  // ── Handlers ──
  function handleSelectPr(pr: typeof prs[number]) {
    autoRunTriggered.current = false;
    resetAnalysis();
    resetFiltering();
    rawSelectPr(pr);
  }

  function goBackToList() {
    autoRunTriggered.current = false;
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
          <button
            className="btn-settings"
            title="Settings"
            onClick={() => setSettingsOpen(true)}
          >
            &#9881;
          </button>
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
            loadingMessage={loading}
            nonSubstantiveHunkIds={nonSubstantiveHunkIds}
            fromCache={fromCache}
            onSelectGroup={setSelectedGroupId}
            onToggleReviewed={toggleReviewed}
            onRunAnalysis={runAnalysis}
            onRefineGroup={refineGroup}
            onBack={goBackToList}
          />
          <DiffPane
            hunks={displayedHunks}
            selectedGroup={selectedGroup}
            selectedGroupId={selectedGroupId}
            nonSubstantiveHunkIds={nonSubstantiveHunkIds}
          />
          <SummaryPane
            selectedPr={selectedPr}
            analysis={analysis}
            selectedGroup={selectedGroup}
            codexLog={codexLog}
            onOpenPr={() => {
              if (selectedPr?.url) {
                openUrl(selectedPr.url).catch((e) => setError(String(e)));
              }
            }}
          />
        </div>
      )}

      {/* Settings Modal — force on first launch */}
      {!hasSettings && (
        <SettingsModal
          initialModel={codexModel}
          initialLang={lang}
          force={true}
          onSave={(s) => {
            saveSettings(s);
          }}
          onClose={() => {}}
        />
      )}

      {/* Settings Modal — edit via gear icon */}
      {settingsOpen && hasSettings && (
        <SettingsModal
          initialModel={codexModel}
          initialLang={lang}
          force={false}
          onSave={(s) => {
            saveSettings(s);
            setSettingsOpen(false);
          }}
          onClose={() => setSettingsOpen(false)}
        />
      )}
    </div>
  );
}

export default App;
