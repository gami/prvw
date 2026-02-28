import type { Hunk, AnalysisResult, IntentGroup } from "../types";
import { UNASSIGNED_GROUP_ID } from "../constants";

interface Props {
  hunks: Hunk[];
  analysis: AnalysisResult | null;
  selectedGroupId: string | null;
  reviewedGroups: Set<string>;
  loading: boolean;
  nonSubstantiveHunkIds: Set<string>;
  fromCache: boolean;
  onSelectGroup: (id: string | null) => void;
  onToggleReviewed: (id: string) => void;
  onRunAnalysis: (force?: boolean) => void;
  onRefineGroup: (group: IntentGroup) => void;
  onBack: () => void;
}

const riskColor = (risk: string) => {
  switch (risk) {
    case "high": return "#e74c3c";
    case "medium": return "#f39c12";
    case "low": return "#27ae60";
    default: return "#888";
  }
};

export function GroupsPane({
  hunks,
  analysis,
  selectedGroupId,
  reviewedGroups,
  loading,
  nonSubstantiveHunkIds,
  fromCache,
  onSelectGroup,
  onToggleReviewed,
  onRunAnalysis,
  onRefineGroup,
  onBack,
}: Props) {
  return (
    <div className="pane pane-left">
      <div className="pane-header">
        <div className="pane-header-row" style={{ display: "flex" }}>
          <h3>Intent Groups</h3>
          <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
            {fromCache && <span className="cache-badge">cached</span>}
            <button
              className="btn btn-accent"
              onClick={() => onRunAnalysis(!!analysis)}
              disabled={loading || hunks.length === 0}
            >
              {analysis ? "Re-run" : "Run"}
            </button>
          </div>
        </div>
        <button className="btn btn-ghost" onClick={onBack}>
          ← Back
        </button>
      </div>
      {!analysis && hunks.length > 0 && (
        <p className="hint">
          {hunks.length} hunks loaded. Click "Run" to group by intent.
        </p>
      )}
      {analysis && (
        <div className="groups-list">
          <div
            className={`group-item ${selectedGroupId === null ? "active" : ""}`}
            onClick={() => onSelectGroup(null)}
          >
            <span className="group-title">All ({hunks.length} hunks)</span>
          </div>
          {analysis.groups.map((g) => (
            <div
              key={g.id}
              className={`group-item ${selectedGroupId === g.id ? "active" : ""} ${reviewedGroups.has(g.id) ? "reviewed" : ""}`}
              onClick={() => onSelectGroup(g.id)}
            >
              <label className="group-check" onClick={(e) => e.stopPropagation()}>
                <input
                  type="checkbox"
                  checked={reviewedGroups.has(g.id)}
                  onChange={() => onToggleReviewed(g.id)}
                />
              </label>
              <div className="group-info">
                <span className="group-title">{g.title}</span>
                <span className="group-meta">
                  <span className="risk-badge" style={{ color: riskColor(g.risk) }}>
                    {g.risk}
                  </span>
                  {" · "}
                  {g.hunkIds.length} hunks
                  {(() => {
                    const c = g.hunkIds.filter((id) => nonSubstantiveHunkIds.has(id)).length;
                    return c > 0 ? ` · ${c} cosmetic` : null;
                  })()}
                  <button
                    className="btn-refine"
                    disabled={loading}
                    onClick={(e) => {
                      e.stopPropagation();
                      onRefineGroup(g);
                    }}
                  >
                    Refine
                  </button>
                </span>
              </div>
            </div>
          ))}
          {analysis.unassignedHunkIds.length > 0 && (
            <div
              className={`group-item ${selectedGroupId === UNASSIGNED_GROUP_ID ? "active" : ""}`}
              onClick={() => onSelectGroup(UNASSIGNED_GROUP_ID)}
            >
              <span className="group-title unassigned">
                Unassigned ({analysis.unassignedHunkIds.length} hunks)
              </span>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
