import { Highlight, themes } from "prism-react-renderer";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import Markdown from "react-markdown";
import { askAboutHunkApi, explainHunkApi } from "../hooks/useCodexApi";
import type { Hunk } from "../types";
import { getFileExtension } from "../utils/fileExtension";

const EXT_TO_LANG: Record<string, string> = {
  ".js": "javascript",
  ".jsx": "jsx",
  ".ts": "typescript",
  ".tsx": "tsx",
  ".go": "go",
  ".rs": "rust",
  ".py": "python",
  ".rb": "ruby",
  ".java": "java",
  ".kt": "kotlin",
  ".swift": "swift",
  ".cs": "csharp",
  ".cpp": "cpp",
  ".c": "c",
  ".h": "c",
  ".css": "css",
  ".scss": "css",
  ".html": "markup",
  ".xml": "markup",
  ".json": "json",
  ".yaml": "yaml",
  ".yml": "yaml",
  ".toml": "toml",
  ".sql": "sql",
  ".sh": "bash",
  ".bash": "bash",
  ".zsh": "bash",
  ".md": "markdown",
  ".graphql": "graphql",
  ".proto": "protobuf",
  ".dockerfile": "docker",
};

function getLang(filePath: string): string {
  const base = filePath.split("/").pop() ?? "";
  if (base === "Dockerfile") return "docker";
  const ext = getFileExtension(filePath).toLowerCase();
  return EXT_TO_LANG[ext] ?? "plaintext";
}

interface Message {
  role: "assistant" | "user";
  content: string;
}

interface Props {
  hunk: Hunk;
  model: string;
  lang: string;
  onClose: () => void;
}

export function HunkDetailModal({ hunk, model, lang, onClose }: Props) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [fromCache, setFromCache] = useState(false);
  const [question, setQuestion] = useState("");
  const [asking, setAsking] = useState(false);
  const scrollRef = useRef<HTMLDivElement>(null);

  const ext = getFileExtension(hunk.filePath);
  const prismLang = getLang(hunk.filePath);

  const scrollToBottom = useCallback(() => {
    requestAnimationFrame(() => {
      scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight, behavior: "smooth" });
    });
  }, []);

  const runExplain = useCallback(
    async (force?: boolean) => {
      setLoading(true);
      setError(null);
      try {
        const res = await explainHunkApi(hunk, model, lang, force);
        setMessages([{ role: "assistant", content: res.explanation }]);
        setFromCache(res.fromCache);
        scrollToBottom();
      } catch (e) {
        setError(String(e));
      } finally {
        setLoading(false);
      }
    },
    [hunk, model, lang, scrollToBottom],
  );

  useEffect(() => {
    runExplain();
  }, [runExplain]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [onClose]);

  const codeLines = useMemo(() => hunk.lines.filter((l) => l.kind === "add" || l.kind === "context"), [hunk.lines]);

  const codeText = useMemo(() => codeLines.map((l) => l.text).join("\n"), [codeLines]);

  // Build context string from all messages for the follow-up prompt
  function buildContext(): string {
    return messages.map((m) => (m.role === "user" ? `Q: ${m.content}` : m.content)).join("\n\n");
  }

  async function handleAsk() {
    const q = question.trim();
    if (!q || asking) return;

    const ctx = buildContext();
    setMessages((prev) => [...prev, { role: "user", content: q }]);
    setQuestion("");
    setAsking(true);
    scrollToBottom();
    setError(null);

    try {
      const res = await askAboutHunkApi(hunk, q, ctx, model, lang);
      setMessages((prev) => [...prev, { role: "assistant", content: res.explanation }]);
      scrollToBottom();
    } catch (e) {
      setError(String(e));
    } finally {
      setAsking(false);
    }
  }

  function handleKeyDown(e: React.KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleAsk();
    }
  }

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="hunk-detail-modal" onClick={(e) => e.stopPropagation()}>
        <div className="hunk-detail-header">
          <button type="button" className="btn-mini" onClick={onClose}>
            ← Back
          </button>
          <div className="hunk-detail-title">
            <span className="hunk-id">{hunk.id}</span>
            <span className="hunk-detail-file">{hunk.filePath}</span>
            <span className="hunk-detail-ext">{ext}</span>
          </div>
          <div className="hunk-detail-actions">
            {fromCache && (
              <button type="button" className="btn-mini" onClick={() => runExplain(true)}>
                Re-run
              </button>
            )}
          </div>
        </div>
        <div className="hunk-detail-body">
          <div className="hunk-detail-code">
            <Highlight theme={themes.nightOwl} code={codeText} language={prismLang}>
              {({ tokens, getTokenProps }) => (
                <pre className="hunk-code hunk-code-detail">
                  {tokens.map((tokenLine, i) => {
                    const srcLine = codeLines[i];
                    const isAdd = srcLine?.kind === "add";
                    return (
                      <div key={i} className={`diff-line${isAdd ? " diff-add-highlight" : ""}`}>
                        <span className="line-num new">{srcLine?.newLine ?? " "}</span>
                        <span className="line-text">
                          {tokenLine.map((token, j) => (
                            <span key={j} {...getTokenProps({ token })} />
                          ))}
                        </span>
                      </div>
                    );
                  })}
                </pre>
              )}
            </Highlight>
          </div>
          <div className="hunk-detail-explain">
            <div className="hunk-detail-messages" ref={scrollRef}>
              {loading && <p className="hint">Analyzing...</p>}
              {messages.map((msg, i) => (
                <div key={i} className={`hunk-msg hunk-msg-${msg.role}`}>
                  {msg.role === "user" ? (
                    <p className="hunk-msg-question">{msg.content}</p>
                  ) : (
                    <div className="hunk-detail-explanation">
                      <Markdown>{msg.content}</Markdown>
                    </div>
                  )}
                </div>
              ))}
              {asking && <p className="hint">Thinking...</p>}
              {error && <p className="error-text">{error}</p>}
            </div>
            <div className="hunk-detail-input">
              <input
                type="text"
                placeholder="Follow-up question..."
                value={question}
                onChange={(e) => setQuestion(e.target.value)}
                onKeyDown={handleKeyDown}
                disabled={loading || asking}
              />
              <button
                type="button"
                className="btn btn-accent"
                onClick={handleAsk}
                disabled={loading || asking || !question.trim()}
              >
                Ask
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
