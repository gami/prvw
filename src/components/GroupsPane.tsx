import { useEffect, useMemo, useState } from "react";
import { UNASSIGNED_GROUP_ID } from "../constants";
import type { AnalysisResult, Hunk, IntentGroup } from "../types";
import { GroupListItem } from "./GroupListItem";

const SPINNER_FRAMES = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];

function useSpinner(active: boolean) {
  const [frame, setFrame] = useState(0);
  useEffect(() => {
    if (!active) return;
    const id = setInterval(() => setFrame((f) => (f + 1) % SPINNER_FRAMES.length), 80);
    return () => clearInterval(id);
  }, [active]);
  return SPINNER_FRAMES[frame];
}

interface Props {
  hunks: Hunk[];
  analysis: AnalysisResult | null;
  selectedGroupId: string | null;
  reviewedGroups: Set<string>;
  loadingMessage: string | null;
  nonSubstantiveHunkIds: Set<string>;
  fromCache: boolean;
  onSelectGroup: (id: string | null) => void;
  onToggleReviewed: (id: string) => void;
  onRunAnalysis: (force?: boolean) => void;
  onRefineGroup: (group: IntentGroup) => void;
  onBack: () => void;
}

export function GroupsPane({
  hunks,
  analysis,
  selectedGroupId,
  reviewedGroups,
  loadingMessage,
  nonSubstantiveHunkIds,
  fromCache,
  onSelectGroup,
  onToggleReviewed,
  onRunAnalysis,
  onRefineGroup,
  onBack,
}: Props) {
  const loading = !!loadingMessage;
  const spinner = useSpinner(loading);

  const reviewProgress = useMemo(() => {
    if (!analysis) return null;
    const total = hunks.length;
    if (total === 0) return null;
    const reviewedCount = analysis.groups
      .filter((g) => reviewedGroups.has(g.id))
      .reduce((sum, g) => sum + g.hunkIds.length, 0);
    const pct = Math.round((reviewedCount / total) * 100);
    return { reviewedCount, total, pct };
  }, [analysis, hunks.length, reviewedGroups]);

  return (
    <div className="pane pane-left">
      <div className="pane-header">
        <div className="pane-header-row" style={{ display: "flex" }}>
          <h3>Intent Groups</h3>
          <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
            {fromCache && (
              <button
                type="button"
                className="cache-badge-btn"
                onClick={() => onRunAnalysis(true)}
                disabled={loading}
                title="Re-run analysis (bypass cache)"
              >
                <span className="cache-badge-label">cached</span>
                <span className="cache-badge-reload">reload</span>
              </button>
            )}
            <button
              type="button"
              className="btn btn-accent"
              onClick={() => onRunAnalysis(!!analysis)}
              disabled={loading || hunks.length === 0}
            >
              {analysis ? "Re-run" : "Run"}
            </button>
          </div>
        </div>
        <button type="button" className="btn btn-ghost" onClick={onBack}>
          ← Back
        </button>
      </div>
      {loading && (
        <div className="spinner-bar">
          <span className="spinner-char">{spinner}</span>
          <span className="spinner-text">{loadingMessage}</span>
        </div>
      )}
      {!analysis && !loading && hunks.length > 0 && (
        <p className="hint">{hunks.length} hunks loaded. Click "Run" to group by intent.</p>
      )}
      {analysis && (
        <div className="groups-list">
          <div
            className={`group-item ${selectedGroupId === null ? "active" : ""}`}
            style={{ alignItems: "center" }}
            onClick={() => onSelectGroup(null)}
          >
            <span className="group-title">All ({hunks.length} hunks)</span>
            {reviewProgress && (
              <span className="review-progress-wrap">
                <span className="review-progress-bar">
                  <span className="review-progress-fill" style={{ width: `${reviewProgress.pct}%` }} />
                </span>
                <span
                  className="review-progress-label"
                  style={reviewProgress.pct === 100 ? { color: "#fff" } : undefined}
                >
                  {reviewProgress.pct}%
                </span>
              </span>
            )}
          </div>
          {analysis.groups.map((g) => (
            <GroupListItem
              key={g.id}
              group={g}
              selected={selectedGroupId === g.id}
              reviewed={reviewedGroups.has(g.id)}
              loading={loading}
              nonSubstantiveHunkIds={nonSubstantiveHunkIds}
              onSelect={() => onSelectGroup(g.id)}
              onToggleReviewed={() => onToggleReviewed(g.id)}
              onRefine={() => onRefineGroup(g)}
            />
          ))}
          {analysis.unassignedHunkIds.length > 0 && (
            <div
              className={`group-item ${selectedGroupId === UNASSIGNED_GROUP_ID ? "active" : ""}`}
              onClick={() => onSelectGroup(UNASSIGNED_GROUP_ID)}
            >
              <span className="group-title unassigned">Unassigned ({analysis.unassignedHunkIds.length} hunks)</span>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
