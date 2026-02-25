import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Hunk, AnalysisResult, AnalysisResponse, SplitResponse } from "../types";

interface UseAnalysisOptions {
  hunks: Hunk[];
  codexModel: string;
  lang: string;
  setHunks: (hunks: Hunk[]) => void;
  setError: (error: string | null) => void;
  setLoading: (msg: string | null) => void;
}

export function useAnalysis({
  hunks,
  codexModel,
  lang,
  setHunks,
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

    let currentHunks = hunks;
    let logs = "";

    // Step 1: Split large hunks (>100 lines) if any exist
    const largeCount = currentHunks.filter((h) => h.lines.length > 100).length;
    if (largeCount > 0) {
      setLoading(`Splitting ${largeCount} large hunk(s) with Codex...`);
      try {
        const splitRes = await invoke<SplitResponse>("split_large_hunks", {
          hunksJson: JSON.stringify(currentHunks),
          model: codexModel.trim() || null,
          lang: lang.trim() || null,
        });
        currentHunks = splitRes.hunks;
        if (splitRes.codexLog) {
          logs += `[split] ${splitRes.codexLog}\n`;
        }
        setHunks(currentHunks);
      } catch (e) {
        setError(`Hunk splitting failed (continuing with original hunks): ${e}`);
      }
    }

    // Step 2: Intent analysis
    setLoading("Running intent analysis with Codex... (this may take a minute)");
    try {
      const res = await invoke<AnalysisResponse>("analyze_intents_with_codex", {
        hunksJson: JSON.stringify(currentHunks),
        model: codexModel.trim() || null,
        lang: lang.trim() || null,
      });
      setAnalysis(res.result);
      if (res.codexLog) {
        logs += `[analysis] ${res.codexLog}\n`;
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(null);
      setCodexLog(logs);
    }
  }

  function resetAnalysis() {
    setAnalysis(null);
    setCodexLog("");
  }

  return { analysis, codexLog, runAnalysis, resetAnalysis };
}
