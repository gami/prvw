import { openUrl } from "@tauri-apps/plugin-opener";
import { useMemo, useState } from "react";
import { DiffPane } from "./components/DiffPane";
import { GroupsPane } from "./components/GroupsPane";
import { Header } from "./components/Header";
import { PrList } from "./components/PrList";
import { SettingsModal } from "./components/SettingsModal";
import { SummaryPane } from "./components/SummaryPane";
import { useAnalysis } from "./hooks/useAnalysis";
import { useAutoRunAnalysis } from "./hooks/useAutoRunAnalysis";
import { useGroupFiltering } from "./hooks/useGroupFiltering";
import { usePrDiff } from "./hooks/usePrDiff";
import { usePrList } from "./hooks/usePrList";
import { useRepoHistory } from "./hooks/useRepoHistory";
import { useSettings } from "./hooks/useSettings";
import "./App.css";

function App() {
  // ── Global UI state ──
  const [search, setSearch] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState<string | null>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);

  // ── Settings ──
  const { codexModel, lang, hasSettings, saveSettings } = useSettings();

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

  const {
    selectedPr,
    hunks,
    selectPr: rawSelectPr,
    clearSelection,
  } = usePrDiff({
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

  const nonSubstantiveHunkIds = useMemo(() => new Set(analysis?.nonSubstantiveHunkIds ?? []), [analysis]);

  // ── Auto-run analysis when hunks are loaded ──
  const { resetAutoRun } = useAutoRunAnalysis({
    hunks,
    analysis,
    loading,
    hasSettings,
    runAnalysis,
  });

  // ── Handlers ──
  function handleSelectPr(pr: (typeof prs)[number]) {
    resetAutoRun();
    resetAnalysis();
    resetFiltering();
    rawSelectPr(pr);
  }

  function goBackToList() {
    resetAutoRun();
    clearSelection();
    resetAnalysis();
    resetFiltering();
  }

  // ── Render ──
  return (
    <div className="app">
      <Header
        repo={repo}
        search={search}
        repoHistory={repoHistory}
        loading={!!loading}
        onRepoChange={setRepo}
        onSearchChange={setSearch}
        onFetchPrs={fetchPrs}
        onOpenSettings={() => setSettingsOpen(true)}
      />

      {/* Error / Loading */}
      {error && (
        <div className="error-bar">
          <span>{error}</span>
          <button type="button" className="btn-close" onClick={() => setError(null)}>
            ×
          </button>
        </div>
      )}
      {loading && <div className="loading-bar">{loading}</div>}

      {/* PR List (shown when no PR selected) */}
      {!selectedPr && prs.length > 0 && <PrList prs={prs} onSelect={handleSelectPr} />}

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
