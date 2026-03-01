import { useRef, useState } from "react";
import type { AnalysisResult, Hunk, IntentGroup } from "../types";
import { analyzeIntents, refineGroupApi } from "./useCodexApi";

interface UseAnalysisOptions {
  hunks: Hunk[];
  prBody: string | null;
  codexModel: string;
  lang: string;
  setError: (error: string | null) => void;
  setLoading: (msg: string | null) => void;
}

export function useAnalysis({ hunks, prBody, codexModel, lang, setError, setLoading }: UseAnalysisOptions) {
  const [analysis, setAnalysis] = useState<AnalysisResult | null>(null);
  const [codexLog, setCodexLog] = useState<string>("");
  const [fromCache, setFromCache] = useState(false);
  const requestIdRef = useRef(0);

  async function runAnalysis(force?: boolean) {
    const id = ++requestIdRef.current;
    setError(null);
    setCodexLog("");
    setFromCache(false);
    if (hunks.length === 0) {
      setError("No hunks to analyze. Select a PR first.");
      return;
    }

    setLoading(
      force
        ? "Re-running intent analysis with Codex (bypassing cache)..."
        : "Running intent analysis with Codex... (this may take a minute)",
    );
    try {
      const res = await analyzeIntents(hunks, codexModel, lang, force, prBody);
      if (id !== requestIdRef.current) return;
      setAnalysis(res.result);
      setCodexLog(res.codexLog);
      setFromCache(res.fromCache);
    } catch (e) {
      if (id !== requestIdRef.current) return;
      setError(String(e));
    } finally {
      if (id === requestIdRef.current) {
        setLoading(null);
      }
    }
  }

  async function refineGroup(group: IntentGroup, force?: boolean) {
    setError(null);
    setLoading(`Refining "${group.title}"...`);
    try {
      const res = await refineGroupApi(hunks, group, codexModel, lang, force);

      // Replace the refined group with its sub-groups using functional setState
      setAnalysis((prev) => {
        if (!prev) return prev;
        const newGroups: IntentGroup[] = [];
        for (const g of prev.groups) {
          if (g.id === group.id) {
            newGroups.push(...res.subGroups);
          } else {
            newGroups.push(g);
          }
        }
        return { ...prev, groups: newGroups };
      });
      setCodexLog((prev) => `${prev}\n${res.codexLog}`);
      if (!res.fromCache) setFromCache(false);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(null);
    }
  }

  function resetAnalysis() {
    requestIdRef.current++;
    setAnalysis(null);
    setCodexLog("");
    setFromCache(false);
  }

  return { analysis, codexLog, fromCache, runAnalysis, refineGroup, resetAnalysis };
}
