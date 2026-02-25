import { useState, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import type {
  PrListItem,
  Hunk,
  ParsedDiff,
  AnalysisResult,
  IntentGroup,
} from "./types";
import "./App.css";

function App() {
  // ── State ──
  const [repo, setRepo] = useState(() => localStorage.getItem("prvw:repo") ?? "");
  const [search, setSearch] = useState("");
  const [prs, setPrs] = useState<PrListItem[]>([]);
  const [selectedPr, setSelectedPr] = useState<PrListItem | null>(null);
  const [hunks, setHunks] = useState<Hunk[]>([]);
  const [_rawDiff, setRawDiff] = useState("");
  const [analysis, setAnalysis] = useState<AnalysisResult | null>(null);
  const [selectedGroupId, setSelectedGroupId] = useState<string | null>(null);
  const [reviewedGroups, setReviewedGroups] = useState<Set<string>>(new Set());
  const [codexModel, setCodexModel] = useState(() => localStorage.getItem("prvw:codexModel") ?? "");
  const [lang, setLang] = useState(() => localStorage.getItem("prvw:lang") ?? "ja");
  const [fileFilters, setFileFilters] = useState({
    generated: true,
    test: true,
    docs: true,
    config: true,
  });
  const [collapsedFiles, setCollapsedFiles] = useState<Set<string>>(new Set());
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState<string | null>(null);

  // ── File category detection ──
  type FileCategory = "generated" | "test" | "docs" | "config" | "src";

  function classifyFile(filePath: string): FileCategory {
    const p = filePath.toLowerCase();
    const base = p.split("/").pop() ?? "";

    // Generated / auto-generated
    if (
      p.includes("generated") ||
      p.includes("__generated__") ||
      p.includes("/gen/") ||
      p.startsWith("gen/") ||
      base === "package-lock.json" ||
      base === "yarn.lock" ||
      base === "pnpm-lock.yaml" ||
      base === "cargo.lock" ||
      base === "go.sum" ||
      base === "poetry.lock" ||
      base === "composer.lock" ||
      base === "gemfile.lock" ||
      p.endsWith(".min.js") ||
      p.endsWith(".min.css") ||
      p.endsWith(".pb.go") ||
      p.endsWith(".pb.ts") ||
      p.endsWith(".g.dart") ||
      p.endsWith(".generated.ts") ||
      p.endsWith(".generated.js")
    ) return "generated";

    // Tests
    if (
      p.includes("__tests__") ||
      p.includes("__test__") ||
      p.includes("/test/") ||
      p.includes("/tests/") ||
      p.includes("/spec/") ||
      p.includes("/specs/") ||
      base.endsWith(".test.ts") ||
      base.endsWith(".test.tsx") ||
      base.endsWith(".test.js") ||
      base.endsWith(".test.jsx") ||
      base.endsWith(".spec.ts") ||
      base.endsWith(".spec.tsx") ||
      base.endsWith(".spec.js") ||
      base.endsWith(".spec.jsx") ||
      base.startsWith("test_") ||
      base.endsWith("_test.go") ||
      base.endsWith("_test.rs") ||
      base.endsWith("_test.py")
    ) return "test";

    // Documentation
    if (
      p.endsWith(".md") ||
      p.endsWith(".mdx") ||
      p.endsWith(".rst") ||
      p.endsWith(".txt") ||
      p.includes("/docs/") ||
      p.includes("/doc/") ||
      base === "changelog" ||
      base === "license" ||
      base === "licence"
    ) return "docs";

    // Config files
    if (
      base.startsWith(".") ||
      base.endsWith(".toml") ||
      base.endsWith(".yaml") ||
      base.endsWith(".yml") ||
      base.endsWith(".json") ||
      base.endsWith(".ini") ||
      base.endsWith(".cfg") ||
      base.endsWith(".conf") ||
      base.endsWith(".config.js") ||
      base.endsWith(".config.ts") ||
      base.endsWith(".config.mjs") ||
      base === "dockerfile" ||
      base === "makefile" ||
      base === "rakefile" ||
      base === "procfile" ||
      p.includes("/.github/") ||
      p.includes("/.circleci/") ||
      p.includes("/.vscode/")
    ) return "config";

    return "src";
  }

  function toggleFilter(key: keyof typeof fileFilters) {
    setFileFilters((prev) => ({ ...prev, [key]: !prev[key] }));
  }

  // ── Derived ──
  const selectedGroup: IntentGroup | null = useMemo(() => {
    if (!analysis || !selectedGroupId) return null;
    return analysis.groups.find((g) => g.id === selectedGroupId) ?? null;
  }, [analysis, selectedGroupId]);

  const displayedHunks: Hunk[] = useMemo(() => {
    let result: Hunk[];
    if (selectedGroupId === "__unassigned" && analysis) {
      result = hunks.filter((h) => analysis.unassignedHunkIds.includes(h.id));
    } else if (selectedGroup) {
      const ids = new Set(selectedGroup.hunkIds);
      result = hunks.filter((h) => ids.has(h.id));
    } else {
      result = hunks;
    }

    // Apply file category filters
    return result.filter((h) => {
      const cat = classifyFile(h.filePath);
      if (cat === "src") return true; // always show source
      return fileFilters[cat];
    });
  }, [hunks, selectedGroup, selectedGroupId, analysis, fileFilters]);

  // Group hunks by file path (preserving order)
  const fileGroups: { filePath: string; hunks: Hunk[] }[] = useMemo(() => {
    const map = new Map<string, Hunk[]>();
    for (const h of displayedHunks) {
      const arr = map.get(h.filePath);
      if (arr) arr.push(h);
      else map.set(h.filePath, [h]);
    }
    return Array.from(map, ([filePath, hunks]) => ({ filePath, hunks }));
  }, [displayedHunks]);

  function toggleFile(filePath: string) {
    setCollapsedFiles((prev) => {
      const next = new Set(prev);
      if (next.has(filePath)) next.delete(filePath);
      else next.add(filePath);
      return next;
    });
  }

  function collapseAll() {
    setCollapsedFiles(new Set(fileGroups.map((g) => g.filePath)));
  }

  function expandAll() {
    setCollapsedFiles(new Set());
  }

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
      setSelectedPr(null);
      setHunks([]);
      setAnalysis(null);
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
    setAnalysis(null);
    setSelectedGroupId(null);
    setReviewedGroups(new Set());
    setLoading("Fetching diff...");
    try {
      const diff = await invoke<string>("get_pr_diff", {
        repo: repo.trim(),
        prNumber: pr.number,
      });
      setRawDiff(diff);
      const parsed = await invoke<ParsedDiff>("parse_diff", {
        diffText: diff,
      });
      setHunks(parsed.hunks);
    } catch (e) {
      setError(String(e));
      setHunks([]);
      setRawDiff("");
    } finally {
      setLoading(null);
    }
  }

  async function runAnalysis() {
    setError(null);
    if (hunks.length === 0) {
      setError("No hunks to analyze. Select a PR first.");
      return;
    }
    localStorage.setItem("prvw:codexModel", codexModel);
    localStorage.setItem("prvw:lang", lang);

    let currentHunks = hunks;

    // Step 1: Split large hunks (>100 lines) if any exist
    const largeCount = currentHunks.filter((h) => h.lines.length > 100).length;
    if (largeCount > 0) {
      setLoading(`Splitting ${largeCount} large hunk(s) with Codex...`);
      try {
        currentHunks = await invoke<Hunk[]>("split_large_hunks", {
          hunksJson: JSON.stringify(currentHunks),
          model: codexModel.trim() || null,
          lang: lang.trim() || null,
        });
        setHunks(currentHunks);
      } catch (e) {
        setError(`Hunk splitting failed (continuing with original hunks): ${e}`);
        // Continue with original hunks
      }
    }

    // Step 2: Intent analysis
    setLoading("Running intent analysis with Codex... (this may take a minute)");
    try {
      const result = await invoke<AnalysisResult>("analyze_intents_with_codex", {
        hunksJson: JSON.stringify(currentHunks),
        model: codexModel.trim() || null,
        lang: lang.trim() || null,
      });
      setAnalysis(result);
      setSelectedGroupId(null);
    } catch (e) {
      setError(String(e));
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
    setRawDiff("");
    setAnalysis(null);
    setSelectedGroupId(null);
  }

  const riskColor = (risk: string) => {
    switch (risk) {
      case "high": return "#e74c3c";
      case "medium": return "#f39c12";
      case "low": return "#27ae60";
      default: return "#888";
    }
  };

  // ── Render ──
  return (
    <div className="app">
      {/* ── Header ── */}
      <header className="header">
        <div className="header-left">
          <strong className="logo">PRVW</strong>
          <input
            className="input repo-input"
            placeholder="owner/repo"
            value={repo}
            onChange={(e) => setRepo(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && fetchPrs()}
          />
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
            <span className="selected-pr">
              #{selectedPr.number} {selectedPr.title}
            </span>
          )}
        </div>
      </header>

      {/* ── Error / Loading ── */}
      {error && (
        <div className="error-bar">
          <span>{error}</span>
          <button className="btn-close" onClick={() => setError(null)}>×</button>
        </div>
      )}
      {loading && <div className="loading-bar">{loading}</div>}

      {/* ── PR List (shown when no PR selected) ── */}
      {!selectedPr && prs.length > 0 && (
        <div className="pr-list">
          <table>
            <thead>
              <tr>
                <th>#</th>
                <th>Title</th>
                <th>Author</th>
                <th>Branch</th>
                <th>Updated</th>
              </tr>
            </thead>
            <tbody>
              {prs.map((pr) => (
                <tr key={pr.number} onClick={() => selectPr(pr)} className="pr-row">
                  <td>{pr.number}</td>
                  <td>{pr.title}</td>
                  <td>{pr.author?.login ?? "—"}</td>
                  <td className="branch-cell">
                    {pr.headRefName && <span className="branch">{pr.headRefName}</span>}
                  </td>
                  <td>{pr.updatedAt ? new Date(pr.updatedAt).toLocaleDateString() : "—"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* ── 3-Pane Layout ── */}
      {selectedPr && (
        <div className="panes">
          {/* Left: Intent Groups */}
          <div className="pane pane-left">
            <div className="pane-header">
              <h3>Intent Groups</h3>
              <div className="model-row">
                <input
                  className="input model-input"
                  placeholder="model (empty=config)"
                  value={codexModel}
                  onChange={(e) => setCodexModel(e.target.value)}
                />
                <input
                  className="input lang-input"
                  placeholder="lang"
                  value={lang}
                  onChange={(e) => setLang(e.target.value)}
                />
                <button className="btn btn-accent" onClick={runAnalysis} disabled={!!loading || hunks.length === 0}>
                  Run
                </button>
              </div>
              <button className="btn btn-ghost" onClick={goBackToList}>
                ← Back
              </button>
            </div>
            {!analysis && hunks.length > 0 && (
              <p className="hint">{hunks.length} hunks loaded. Click "Run Codex Analysis" to group by intent.</p>
            )}
            {analysis && (
              <div className="groups-list">
                <div
                  className={`group-item ${selectedGroupId === null ? "active" : ""}`}
                  onClick={() => setSelectedGroupId(null)}
                >
                  <span className="group-title">All ({hunks.length} hunks)</span>
                </div>
                {analysis.groups.map((g) => (
                  <div
                    key={g.id}
                    className={`group-item ${selectedGroupId === g.id ? "active" : ""} ${reviewedGroups.has(g.id) ? "reviewed" : ""}`}
                    onClick={() => setSelectedGroupId(g.id)}
                  >
                    <label className="group-check" onClick={(e) => e.stopPropagation()}>
                      <input
                        type="checkbox"
                        checked={reviewedGroups.has(g.id)}
                        onChange={() => toggleReviewed(g.id)}
                      />
                    </label>
                    <div className="group-info">
                      <span className="group-title">{g.title}</span>
                      <span className="group-meta">
                        <span className="risk-badge" style={{ color: riskColor(g.risk) }}>{g.risk}</span>
                        {" · "}{g.hunkIds.length} hunks
                      </span>
                    </div>
                  </div>
                ))}
                {analysis.unassignedHunkIds.length > 0 && (
                  <div
                    className={`group-item ${selectedGroupId === "__unassigned" ? "active" : ""}`}
                    onClick={() => setSelectedGroupId("__unassigned")}
                  >
                    <span className="group-title unassigned">
                      Unassigned ({analysis.unassignedHunkIds.length} hunks)
                    </span>
                  </div>
                )}
              </div>
            )}
          </div>

          {/* Center: Diff View */}
          <div className="pane pane-center">
            <div className="pane-header pane-header-row">
              <h3>
                Diff
                {selectedGroup
                  ? ` — ${selectedGroup.title}`
                  : selectedGroupId === "__unassigned"
                  ? " — Unassigned"
                  : " — All"}
              </h3>
              <div className="file-filters">
                <button className="btn-mini" onClick={expandAll}>Expand all</button>
                <button className="btn-mini" onClick={collapseAll}>Collapse all</button>
                <span className="filter-sep" />
                {(["generated", "test", "docs", "config"] as const).map((key) => (
                  <label key={key} className={`filter-toggle ${fileFilters[key] ? "" : "off"}`}>
                    <input
                      type="checkbox"
                      checked={fileFilters[key]}
                      onChange={() => toggleFilter(key)}
                    />
                    {key}
                  </label>
                ))}
              </div>
            </div>
            <div className="diff-view">
              {fileGroups.map(({ filePath, hunks: fileHunks }) => {
                const collapsed = collapsedFiles.has(filePath);
                const adds = fileHunks.reduce((n, h) => n + h.lines.filter((l) => l.kind === "add").length, 0);
                const dels = fileHunks.reduce((n, h) => n + h.lines.filter((l) => l.kind === "remove").length, 0);
                return (
                  <div key={filePath} className="file-group">
                    <div className="file-header" onClick={() => toggleFile(filePath)}>
                      <span className={`file-chevron ${collapsed ? "collapsed" : ""}`}>&#9662;</span>
                      <span className="file-name">{filePath}</span>
                      <span className="file-stats">
                        {adds > 0 && <span className="stat-add">+{adds}</span>}
                        {dels > 0 && <span className="stat-del">-{dels}</span>}
                      </span>
                      <span className="file-hunk-count">{fileHunks.length} hunk{fileHunks.length !== 1 ? "s" : ""}</span>
                    </div>
                    {!collapsed && fileHunks.map((hunk) => (
                      <div key={hunk.id} className="hunk-block">
                        <div className="hunk-header">
                          <span className="hunk-id">{hunk.id}</span>
                          <span className="hunk-range">{hunk.header}</span>
                        </div>
                        <pre className="hunk-code">
                          {hunk.lines.map((line, i) => (
                            <div key={i} className={`diff-line diff-${line.kind}`}>
                              <span className="line-num old">{line.oldLine ?? " "}</span>
                              <span className="line-num new">{line.newLine ?? " "}</span>
                              <span className="line-prefix">
                                {line.kind === "add" ? "+" : line.kind === "remove" ? "-" : " "}
                              </span>
                              <span className="line-text">{line.text}</span>
                            </div>
                          ))}
                        </pre>
                      </div>
                    ))}
                  </div>
                );
              })}
              {hunks.length === 0 && <p className="hint">No diff loaded.</p>}
            </div>
          </div>

          {/* Right: Summary */}
          <div className="pane pane-right">
            <div className="pane-header">
              <h3>Summary</h3>
            </div>
            {analysis ? (
              <div className="summary-content">
                <section>
                  <h4>Overall Summary</h4>
                  <p>{analysis.overallSummary}</p>
                </section>
                {selectedGroup && (
                  <>
                    <section>
                      <h4>Rationale</h4>
                      <p>{selectedGroup.rationale}</p>
                    </section>
                    {selectedGroup.reviewerChecklist.length > 0 && (
                      <section>
                        <h4>Reviewer Checklist</h4>
                        <ul>
                          {selectedGroup.reviewerChecklist.map((c, i) => (
                            <li key={i}>{c}</li>
                          ))}
                        </ul>
                      </section>
                    )}
                    {selectedGroup.suggestedTests.length > 0 && (
                      <section>
                        <h4>Suggested Tests</h4>
                        <ul>
                          {selectedGroup.suggestedTests.map((t, i) => (
                            <li key={i}>{t}</li>
                          ))}
                        </ul>
                      </section>
                    )}
                  </>
                )}
                {analysis.questions.length > 0 && (
                  <section>
                    <h4>Questions</h4>
                    <ul>
                      {analysis.questions.map((q, i) => (
                        <li key={i}>{q}</li>
                      ))}
                    </ul>
                  </section>
                )}
              </div>
            ) : (
              <p className="hint">Run analysis to see summary, rationale, and checklists.</p>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

export default App;
