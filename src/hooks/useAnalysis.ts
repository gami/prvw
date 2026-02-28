import { useState } from "react";
import type { Hunk, AnalysisResult, IntentGroup } from "../types";
import { analyzeIntents, refineGroupApi } from "./useCodexApi";

interface UseAnalysisOptions {
  hunks: Hunk[];
  codexModel: string;
  lang: string;
  setError: (error: string | null) => void;
  setLoading: (msg: string | null) => void;
}

export function useAnalysis({
  hunks,
  codexModel,
  lang,
  setError,
  setLoading,
}: UseAnalysisOptions) {
  const [analysis, setAnalysis] = useState<AnalysisResult | null>(null);
  const [codexLog, setCodexLog] = useState<string>("");
  const [fromCache, setFromCache] = useState(false);

  async function runAnalysis(force?: boolean) {
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
      const res = await analyzeIntents(hunks, codexModel, lang, force);
      setAnalysis(res.result);
      setCodexLog(res.codexLog);
      setFromCache(res.fromCache);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(null);
    }
  }

  async function refineGroup(group: IntentGroup) {
    setError(null);
    setLoading(`Refining "${group.title}"...`);
    try {
      const res = await refineGroupApi(hunks, group, codexModel, lang);

      if (!analysis) return;

      // Replace the refined group with its sub-groups
      const newGroups: IntentGroup[] = [];
      for (const g of analysis.groups) {
        if (g.id === group.id) {
          newGroups.push(...res.subGroups);
        } else {
          newGroups.push(g);
        }
      }
      setAnalysis({ ...analysis, groups: newGroups });
      setCodexLog((prev) => prev + "\n" + res.codexLog);
      if (!res.fromCache) setFromCache(false);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(null);
    }
  }

  function resetAnalysis() {
    setAnalysis(null);
    setCodexLog("");
    setFromCache(false);
  }

  return { analysis, codexLog, fromCache, runAnalysis, refineGroup, resetAnalysis };
}
