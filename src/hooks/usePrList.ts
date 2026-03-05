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
  const loadingMoreRef = useRef(false);
  const activeQueryRef = useRef({ repo: "", search: "" });

  async function fetchPrs() {
    limitRef.current = PAGE_SIZE;
    setHasMore(true);
    const id = ++requestIdRef.current;
    setError(null);
    if (!repo.trim()) {
      setError("Please enter a repository (owner/repo).");
      return;
    }
    const trimmedRepo = repo.trim();
    const trimmedSearch = search.trim() || null;
    activeQueryRef.current = { repo: trimmedRepo, search: trimmedSearch ?? "" };
    setLoading("Fetching PRs...");
    try {
      const items = await invoke<PrListItem[]>("list_prs", {
        repo: trimmedRepo,
        limit: PAGE_SIZE,
        state: "open",
        search: trimmedSearch,
      });
      if (id !== requestIdRef.current) return;
      setPrs(items);
      onFetched(trimmedRepo);
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
    if (!hasMore || loadingMoreRef.current) return;
    loadingMoreRef.current = true;
    setLoadingMore(true);
    const id = ++requestIdRef.current;
    limitRef.current += PAGE_SIZE;
    const { repo: activeRepo, search: activeSearch } = activeQueryRef.current;
    try {
      const items = await invoke<PrListItem[]>("list_prs", {
        repo: activeRepo,
        limit: limitRef.current,
        state: "open",
        search: activeSearch || null,
      });
      if (id !== requestIdRef.current) return;
      setPrs(items);
      if (items.length < limitRef.current) {
        setHasMore(false);
      }
    } catch (e) {
      if (id !== requestIdRef.current) return;
      setError(String(e));
    } finally {
      if (id === requestIdRef.current) {
        loadingMoreRef.current = false;
        setLoadingMore(false);
      }
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
