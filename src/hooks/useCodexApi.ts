import { invoke } from "@tauri-apps/api/core";
import type { AnalysisResponse, ExplainResponse, Hunk, IntentGroup, RefineResponse } from "../types";

export async function analyzeIntents(
  hunks: Hunk[],
  model: string,
  lang: string,
  force?: boolean,
  prBody?: string | null,
): Promise<AnalysisResponse> {
  return invoke<AnalysisResponse>("analyze_intents_with_codex", {
    hunksJson: JSON.stringify(hunks),
    prBody: prBody || null,
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

export async function explainHunkApi(
  hunk: Hunk,
  model: string,
  lang: string,
  force?: boolean,
): Promise<ExplainResponse> {
  return invoke<ExplainResponse>("explain_hunk", {
    hunkJson: JSON.stringify(hunk),
    filePath: hunk.filePath,
    model: model.trim() || null,
    lang: lang.trim() || null,
    force: force ?? false,
  });
}

export async function askAboutHunkApi(
  hunk: Hunk,
  question: string,
  context: string,
  model: string,
  lang: string,
): Promise<ExplainResponse> {
  return invoke<ExplainResponse>("ask_about_hunk", {
    hunkJson: JSON.stringify(hunk),
    filePath: hunk.filePath,
    question,
    context,
    model: model.trim() || null,
    lang: lang.trim() || null,
  });
}
