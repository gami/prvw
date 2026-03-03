import { invoke } from "@tauri-apps/api/core";
import { useRef, useState } from "react";
import type { Hunk, ParsedDiff, PrListItem } from "../types";

interface UsePrDiffOptions {
  repo: string;
  setError: (error: string | null) => void;
  setLoading: (msg: string | null) => void;
}

export function usePrDiff({ repo, setError, setLoading }: UsePrDiffOptions) {
  const [selectedPr, setSelectedPr] = useState<PrListItem | null>(null);
  const [hunks, setHunks] = useState<Hunk[]>([]);
  const requestIdRef = useRef(0);

  async function selectPr(pr: PrListItem) {
    const id = ++requestIdRef.current;
    setError(null);
    setSelectedPr(pr);
    setLoading("Fetching diff...");
    try {
      const diff = await invoke<string>("get_pr_diff", {
        repo: repo.trim(),
        prNumber: pr.number,
        updatedAt: pr.updatedAt,
      });
      if (id !== requestIdRef.current) return;
      const parsed = await invoke<ParsedDiff>("parse_diff", {
        diffText: diff,
      });
      if (id !== requestIdRef.current) return;
      setHunks(parsed.hunks);
    } catch (e) {
      if (id !== requestIdRef.current) return;
      setError(String(e));
      setHunks([]);
    } finally {
      if (id === requestIdRef.current) {
        setLoading(null);
      }
    }
  }

  function clearSelection() {
    requestIdRef.current++;
    setSelectedPr(null);
    setHunks([]);
  }

  return { selectedPr, hunks, selectPr, clearSelection };
}
