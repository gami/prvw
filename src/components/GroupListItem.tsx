import type { IntentGroup } from "../types";
import { riskColor } from "../utils/riskColor";

interface Props {
  group: IntentGroup;
  selected: boolean;
  reviewed: boolean;
  loading: boolean;
  nonSubstantiveHunkIds: Set<string>;
  onSelect: () => void;
  onToggleReviewed: () => void;
  onRefine: () => void;
}

export function GroupListItem({
  group,
  selected,
  reviewed,
  loading,
  nonSubstantiveHunkIds,
  onSelect,
  onToggleReviewed,
  onRefine,
}: Props) {
  const cosmeticCount = group.hunkIds.filter((id) => nonSubstantiveHunkIds.has(id)).length;

  return (
    <div className={`group-item ${selected ? "active" : ""} ${reviewed ? "reviewed" : ""}`} onClick={onSelect}>
      <label className="group-check" onClick={(e) => e.stopPropagation()}>
        <input type="checkbox" checked={reviewed} onChange={onToggleReviewed} />
      </label>
      <div className="group-info">
        <span className="group-title">{group.title}</span>
        <span className="group-meta">
          {group.category && (
            <span
              className="category-badge"
              style={{ borderColor: riskColor(group.risk), color: riskColor(group.risk) }}
            >
              {group.category}
            </span>
          )}
          {" · "}
          {group.hunkIds.length} hunks
          {cosmeticCount > 0 ? ` · ${cosmeticCount} cosmetic` : null}
          <button
            type="button"
            className="btn-refine"
            disabled={loading}
            onClick={(e) => {
              e.stopPropagation();
              onRefine();
            }}
          >
            Refine
          </button>
        </span>
      </div>
    </div>
  );
}
