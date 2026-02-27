import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
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
      onFetched(repo.trim());
      if (items.length === 0) {
        setError("No open PRs found.");
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(null);
    }
  }

  // Auto-fetch on startup if repo is saved
  useEffect(() => {
    if (repo.trim()) {
      fetchPrs();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return { prs, fetchPrs };
}
