import { useState } from "react";
import type { AnalysisResult, IntentGroup } from "../types";

interface Props {
  analysis: AnalysisResult | null;
  selectedGroup: IntentGroup | null;
  codexLog: string;
}

export function SummaryPane({ analysis, selectedGroup, codexLog }: Props) {
  const [logOpen, setLogOpen] = useState(false);

  return (
    <div className="pane pane-right">
      {analysis ? (
        <div className="summary-content">
          {/* ── Summary ── */}
          <div className="summary-card">
            <div className="summary-card-header">Summary</div>
            <div className="summary-card-body">
              <p>{analysis.overallSummary}</p>
            </div>
          </div>

          {/* ── Group ── */}
          <div className="summary-card">
            <div className="summary-card-header">
              Intent Group{selectedGroup ? ` — ${selectedGroup.title}` : ""}
            </div>
            <div className="summary-card-body">
              {selectedGroup ? (
                <>
                  <section>
                    <h4>Rationale</h4>
                    <p>{selectedGroup.rationale}</p>
                  </section>
                  {selectedGroup.reviewerChecklist.length > 0 && (
                    <section>
                      <h4>Reviewer Checklist</h4>
                      <ul>
                        {selectedGroup.reviewerChecklist.map((c, i) => (
                          <li key={i}>{c}</li>
                        ))}
                      </ul>
                    </section>
                  )}
                  {selectedGroup.suggestedTests.length > 0 && (
                    <section>
                      <h4>Suggested Tests</h4>
                      <ul>
                        {selectedGroup.suggestedTests.map((t, i) => (
                          <li key={i}>{t}</li>
                        ))}
                      </ul>
                    </section>
                  )}
                </>
              ) : (
                <p className="hint">Select a group to see details.</p>
              )}
            </div>
          </div>

          {/* ── AI Comments ── */}
          <div className="summary-card">
            <div className="summary-card-header">AI Comments</div>
            <div className="summary-card-body">
              {analysis.questions.length > 0 && (
                <section>
                  <h4>Questions</h4>
                  <ul>
                    {analysis.questions.map((q, i) => (
                      <li key={i}>{q}</li>
                    ))}
                  </ul>
                </section>
              )}
              {codexLog && (
                <section>
                  <h4
                    className="codex-log-toggle"
                    onClick={() => setLogOpen((v) => !v)}
                  >
                    Codex Log {logOpen ? "▾" : "▸"}
                  </h4>
                  {logOpen && <pre className="codex-log">{codexLog}</pre>}
                </section>
              )}
              {analysis.questions.length === 0 && !codexLog && (
                <p className="hint">No comments.</p>
              )}
            </div>
          </div>
        </div>
      ) : (
        <p className="hint" style={{ padding: 16 }}>Run analysis to see summary, rationale, and checklists.</p>
      )}
    </div>
  );
}
