import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";
import type { PrListItem } from "../types";

interface UsePrListOptions {
  repo: string;
  search: string;
  onFetched: (repo: string) => void;
  setError: (error: string | null) => void;
  setLoading: (msg: string | null) => void;
}

export function usePrList({ repo, search, onFetched, setError, setLoading }: UsePrListOptions) {
  const [prs, setPrs] = useState<PrListItem[]>([]);
  const requestIdRef = useRef(0);

  async function fetchPrs() {
    const id = ++requestIdRef.current;
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
      if (id !== requestIdRef.current) return;
      setPrs(items);
      onFetched(repo.trim());
      if (items.length === 0) {
        setError("No open PRs found.");
      }
    } catch (e) {
      if (id !== requestIdRef.current) return;
      setError(String(e));
    } finally {
      if (id === requestIdRef.current) {
        setLoading(null);
      }
    }
  }

  // biome-ignore lint/correctness/useExhaustiveDependencies: run only once on mount
  useEffect(() => {
    if (repo.trim()) {
      fetchPrs();
    }
  }, []);

  return { prs, fetchPrs };
}
