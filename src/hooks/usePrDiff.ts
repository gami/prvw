import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { PrListItem, Hunk, ParsedDiff } from "../types";

interface UsePrDiffOptions {
  repo: string;
  setError: (error: string | null) => void;
  setLoading: (msg: string | null) => void;
}

export function usePrDiff({ repo, setError, setLoading }: UsePrDiffOptions) {
  const [selectedPr, setSelectedPr] = useState<PrListItem | null>(null);
  const [hunks, setHunks] = useState<Hunk[]>([]);

  async function selectPr(pr: PrListItem) {
    setError(null);
    setSelectedPr(pr);
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

  function clearSelection() {
    setSelectedPr(null);
    setHunks([]);
  }

  return { selectedPr, hunks, selectPr, clearSelection };
}
