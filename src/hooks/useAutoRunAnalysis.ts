import { useEffect, useRef } from "react";
import type { AnalysisResult, Hunk } from "../types";

interface Options {
  hunks: Hunk[];
  analysis: AnalysisResult | null;
  loading: string | null;
  hasSettings: boolean;
  runAnalysis: (force?: boolean) => void;
}

export function useAutoRunAnalysis({ hunks, analysis, loading, hasSettings, runAnalysis }: Options) {
  const triggered = useRef(false);

  // biome-ignore lint/correctness/useExhaustiveDependencies: runAnalysis intentionally excluded to avoid re-trigger loops
  useEffect(() => {
    if (hunks.length > 0 && !analysis && !loading && hasSettings && !triggered.current) {
      triggered.current = true;
      runAnalysis();
    }
  }, [hunks, analysis, loading, hasSettings]);

  function reset() {
    triggered.current = false;
  }

  return { resetAutoRun: reset };
}
