import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";
import type { PrListItem } from "../types";

const PAGE_SIZE = 30;

interface UsePrListOptions {
  repo: string;
  search: string;
  onFetched: (repo: string) => void;
  setError: (error: string | null) => void;
  setLoading: (msg: string | null) => void;
}

export function usePrList({ repo, search, onFetched, setError, setLoading }: UsePrListOptions) {
  const [prs, setPrs] = useState<PrListItem[]>([]);
  const [hasMore, setHasMore] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const requestIdRef = useRef(0);
  const limitRef = useRef(PAGE_SIZE);

  async function fetchPrs() {
    limitRef.current = PAGE_SIZE;
    setHasMore(true);
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
        limit: PAGE_SIZE,
        state: "open",
        search: search.trim() || null,
      });
      if (id !== requestIdRef.current) return;
      setPrs(items);
      onFetched(repo.trim());
      if (items.length < PAGE_SIZE) {
        setHasMore(false);
      }
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

  async function fetchMore() {
    if (!hasMore || loadingMore) return;
    setLoadingMore(true);
    limitRef.current += PAGE_SIZE;
    try {
      const items = await invoke<PrListItem[]>("list_prs", {
        repo: repo.trim(),
        limit: limitRef.current,
        state: "open",
        search: search.trim() || null,
      });
      setPrs(items);
      if (items.length < limitRef.current) {
        setHasMore(false);
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setLoadingMore(false);
    }
  }

  // biome-ignore lint/correctness/useExhaustiveDependencies: run only once on mount
  useEffect(() => {
    if (repo.trim()) {
      fetchPrs();
    }
  }, []);

  return { prs, fetchPrs, fetchMore, hasMore, loadingMore };
}
