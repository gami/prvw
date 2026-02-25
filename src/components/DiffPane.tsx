import { useState, useMemo } from "react";
import type { Hunk, IntentGroup } from "../types";
import type { FileCategory } from "../utils/classifyFile";
import { classifyFile } from "../utils/classifyFile";

interface Props {
  hunks: Hunk[];
  selectedGroup: IntentGroup | null;
  selectedGroupId: string | null;
}

export function DiffPane({ hunks, selectedGroup, selectedGroupId }: Props) {
  const [fileFilters, setFileFilters] = useState({
    generated: true,
    test: true,
    docs: true,
    config: true,
  });
  const [collapsedFiles, setCollapsedFiles] = useState<Set<string>>(new Set());

  function toggleFilter(key: keyof typeof fileFilters) {
    setFileFilters((prev) => ({ ...prev, [key]: !prev[key] }));
  }

  const filteredHunks = useMemo(() => {
    return hunks.filter((h) => {
      const cat: FileCategory = classifyFile(h.filePath);
      if (cat === "src") return true;
      return fileFilters[cat];
    });
  }, [hunks, fileFilters]);

  const fileGroups = useMemo(() => {
    const map = new Map<string, Hunk[]>();
    for (const h of filteredHunks) {
      const arr = map.get(h.filePath);
      if (arr) arr.push(h);
      else map.set(h.filePath, [h]);
    }
    return Array.from(map, ([filePath, hunks]) => ({ filePath, hunks }));
  }, [filteredHunks]);

  function toggleFile(filePath: string) {
    setCollapsedFiles((prev) => {
      const next = new Set(prev);
      if (next.has(filePath)) next.delete(filePath);
      else next.add(filePath);
      return next;
    });
  }

  function collapseAll() {
    setCollapsedFiles(new Set(fileGroups.map((g) => g.filePath)));
  }

  function expandAll() {
    setCollapsedFiles(new Set());
  }

  const title = selectedGroup
    ? ` — ${selectedGroup.title}`
    : selectedGroupId === "__unassigned"
    ? " — Unassigned"
    : " — All";

  return (
    <div className="pane pane-center">
      <div className="pane-header pane-header-row">
        <h3>Diff{title}</h3>
        <div className="file-filters">
          <button className="btn-mini" onClick={expandAll}>Expand all</button>
          <button className="btn-mini" onClick={collapseAll}>Collapse all</button>
          <span className="filter-sep" />
          {(["generated", "test", "docs", "config"] as const).map((key) => (
            <label key={key} className={`filter-toggle ${fileFilters[key] ? "" : "off"}`}>
              <input
                type="checkbox"
                checked={fileFilters[key]}
                onChange={() => toggleFilter(key)}
              />
              {key}
            </label>
          ))}
        </div>
      </div>
      <div className="diff-view">
        {fileGroups.map(({ filePath, hunks: fileHunks }) => {
          const collapsed = collapsedFiles.has(filePath);
          const adds = fileHunks.reduce(
            (n, h) => n + h.lines.filter((l) => l.kind === "add").length,
            0,
          );
          const dels = fileHunks.reduce(
            (n, h) => n + h.lines.filter((l) => l.kind === "remove").length,
            0,
          );
          return (
            <div key={filePath} className="file-group">
              <div className="file-header" onClick={() => toggleFile(filePath)}>
                <span className={`file-chevron ${collapsed ? "collapsed" : ""}`}>&#9662;</span>
                <span className="file-name">{filePath}</span>
                <span className="file-stats">
                  {adds > 0 && <span className="stat-add">+{adds}</span>}
                  {dels > 0 && <span className="stat-del">-{dels}</span>}
                </span>
                <span className="file-hunk-count">
                  {fileHunks.length} hunk{fileHunks.length !== 1 ? "s" : ""}
                </span>
              </div>
              {!collapsed &&
                fileHunks.map((hunk) => (
                  <div key={hunk.id} className="hunk-block">
                    <div className="hunk-header">
                      <span className="hunk-id">{hunk.id}</span>
                      <span className="hunk-range">{hunk.header}</span>
                    </div>
                    <pre className="hunk-code">
                      {hunk.lines.map((line, i) => (
                        <div key={i} className={`diff-line diff-${line.kind}`}>
                          <span className="line-num old">{line.oldLine ?? " "}</span>
                          <span className="line-num new">{line.newLine ?? " "}</span>
                          <span className="line-prefix">
                            {line.kind === "add" ? "+" : line.kind === "remove" ? "-" : " "}
                          </span>
                          <span className="line-text">{line.text}</span>
                        </div>
                      ))}
                    </pre>
                  </div>
                ))}
            </div>
          );
        })}
        {hunks.length === 0 && <p className="hint">No diff loaded.</p>}
      </div>
    </div>
  );
}
