import { useState, useMemo } from "react";
import type { Hunk, AnalysisResult, IntentGroup } from "../types";
import { UNASSIGNED_GROUP_ID } from "../constants";

export function useGroupFiltering(hunks: Hunk[], analysis: AnalysisResult | null) {
  const [selectedGroupId, setSelectedGroupId] = useState<string | null>(null);
  const [reviewedGroups, setReviewedGroups] = useState<Set<string>>(new Set());

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

  function toggleReviewed(groupId: string) {
    setReviewedGroups((prev) => {
      const next = new Set(prev);
      if (next.has(groupId)) next.delete(groupId);
      else next.add(groupId);
      return next;
    });
  }

  function resetFiltering() {
    setSelectedGroupId(null);
    setReviewedGroups(new Set());
  }

  return {
    selectedGroupId,
    setSelectedGroupId,
    selectedGroup,
    displayedHunks,
    reviewedGroups,
    toggleReviewed,
    resetFiltering,
  };
}
