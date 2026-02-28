export interface PrAuthor {
  login: string;
}

export interface PrListItem {
  number: number;
  title: string;
  url: string;
  updatedAt: string;
  author: PrAuthor | null;
  headRefName: string | null;
  baseRefName: string | null;
  reviewDecision: string | null;
  body: string | null;
}

export interface DiffLine {
  kind: "add" | "remove" | "context";
  oldLine: number | null;
  newLine: number | null;
  text: string;
}

export interface Hunk {
  id: string;
  filePath: string;
  header: string;
  oldStart: number;
  oldLines: number;
  newStart: number;
  newLines: number;
  lines: DiffLine[];
}

export interface ParsedDiff {
  hunks: Hunk[];
}

export type GroupCategory = "schema" | "logic" | "api" | "ui" | "test" | "config" | "docs" | "refactor" | "other";

export interface IntentGroup {
  id: string;
  title: string;
  category: GroupCategory;
  rationale: string;
  risk: "low" | "medium" | "high";
  hunkIds: string[];
  reviewerChecklist: string[];
  suggestedTests: string[];
}

export interface AnalysisResult {
  version: number;
  overallSummary: string;
  groups: IntentGroup[];
  unassignedHunkIds: string[];
  nonSubstantiveHunkIds: string[];
  questions: string[];
}

export interface AnalysisResponse {
  result: AnalysisResult;
  codexLog: string;
  fromCache: boolean;
}

export interface RefineResponse {
  subGroups: IntentGroup[];
  codexLog: string;
  fromCache: boolean;
}
