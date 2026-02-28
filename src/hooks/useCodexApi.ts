import { invoke } from "@tauri-apps/api/core";
import type { Hunk, AnalysisResponse, IntentGroup, RefineResponse } from "../types";

export async function analyzeIntents(
  hunks: Hunk[],
  model: string,
  lang: string,
  force?: boolean,
): Promise<AnalysisResponse> {
  return invoke<AnalysisResponse>("analyze_intents_with_codex", {
    hunksJson: JSON.stringify(hunks),
    model: model.trim() || null,
    lang: lang.trim() || null,
    force: force ?? false,
  });
}

export async function refineGroupApi(
  hunks: Hunk[],
  group: IntentGroup,
  model: string,
  lang: string,
  force?: boolean,
): Promise<RefineResponse> {
  return invoke<RefineResponse>("refine_group", {
    hunksJson: JSON.stringify(hunks),
    groupId: group.id,
    groupTitle: group.title,
    hunkIds: group.hunkIds,
    model: model.trim() || null,
    lang: lang.trim() || null,
    force: force ?? false,
  });
}
