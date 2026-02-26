import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Hunk, AnalysisResult, AnalysisResponse, IntentGroup, RefineResponse } from "../types";

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
    localStorage.setItem("prvw:codexModel", codexModel);
    localStorage.setItem("prvw:lang", lang);

    setLoading("Running intent analysis with Codex... (this may take a minute)");
    try {
      const res = await invoke<AnalysisResponse>("analyze_intents_with_codex", {
        hunksJson: JSON.stringify(hunks),
        model: codexModel.trim() || null,
        lang: lang.trim() || null,
      });
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
      const res = await invoke<RefineResponse>("refine_group", {
        hunksJson: JSON.stringify(hunks),
        groupId: group.id,
        groupTitle: group.title,
        hunkIds: group.hunkIds,
        model: codexModel.trim() || null,
        lang: lang.trim() || null,
      });

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
