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

  async function runAnalysis() {
    setError(null);
    setCodexLog("");
    if (hunks.length === 0) {
      setError("No hunks to analyze. Select a PR first.");
      return;
    }

    setLoading("Running intent analysis with Codex... (this may take a minute)");
    try {
      const res = await analyzeIntents(hunks, codexModel, lang);
      setAnalysis(res.result);
      setCodexLog(res.codexLog);
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
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(null);
    }
  }

  function resetAnalysis() {
    setAnalysis(null);
    setCodexLog("");
  }

  return { analysis, codexLog, runAnalysis, refineGroup, resetAnalysis };
}
