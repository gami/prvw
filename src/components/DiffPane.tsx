import { useMemo, useState } from "react";
import { UNASSIGNED_GROUP_ID } from "../constants";
import type { Hunk, IntentGroup } from "../types";
import { classifyFile } from "../utils/classifyFile";
import { getFileExtension } from "../utils/fileExtension";

interface Props {
  hunks: Hunk[];
  selectedGroup: IntentGroup | null;
  selectedGroupId: string | null;
  nonSubstantiveHunkIds: Set<string>;
}

export function DiffPane({ hunks, selectedGroup, selectedGroupId, nonSubstantiveHunkIds }: Props) {
  const [hideTests, setHideTests] = useState(false);
  const [hiddenExts, setHiddenExts] = useState<Set<string>>(new Set());
  const [collapseCosmetic, setCollapseCosmetic] = useState(true);
  const [collapsedFiles, setCollapsedFiles] = useState<Set<string>>(new Set());

  const allExtensions = useMemo(() => {
    const exts = new Set<string>();
    for (const h of hunks) {
      exts.add(getFileExtension(h.filePath));
    }
    return Array.from(exts).sort();
  }, [hunks]);

  function toggleExt(ext: string) {
    setHiddenExts((prev) => {
      const next = new Set(prev);
      if (next.has(ext)) next.delete(ext);
      else next.add(ext);
      return next;
    });
  }

  const hasNonSubstantive = nonSubstantiveHunkIds.size > 0;

  const filteredHunks = useMemo(() => {
    return hunks.filter((h) => {
      if (hideTests && classifyFile(h.filePath) === "test") return false;
      if (hiddenExts.has(getFileExtension(h.filePath))) return false;
      return true;
    });
  }, [hunks, hideTests, hiddenExts]);

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
    : selectedGroupId === UNASSIGNED_GROUP_ID
      ? " — Unassigned"
      : " — All";

  return (
    <div className="pane pane-center">
      <div className="pane-header pane-header-row">
        <h3>Diff{title}</h3>
        <div className="file-filters">
          <button type="button" className="btn-mini" onClick={expandAll}>
            Expand all
          </button>
          <button type="button" className="btn-mini" onClick={collapseAll}>
            Collapse all
          </button>
          <span className="filter-sep" />
          <label className={`filter-toggle ${hideTests ? "off" : ""}`}>
            <input type="checkbox" checked={!hideTests} onChange={() => setHideTests((v) => !v)} />
            test
          </label>
          {allExtensions.map((ext) => (
            <label key={ext} className={`filter-toggle ${hiddenExts.has(ext) ? "off" : ""}`}>
              <input type="checkbox" checked={!hiddenExts.has(ext)} onChange={() => toggleExt(ext)} />
              {ext}
            </label>
          ))}
          {hasNonSubstantive && (
            <>
              <span className="filter-sep" />
              <label className={`filter-toggle ${collapseCosmetic ? "filter-active" : "off"}`}>
                <input type="checkbox" checked={collapseCosmetic} onChange={() => setCollapseCosmetic((v) => !v)} />
                hide cosmetic
              </label>
            </>
          )}
        </div>
      </div>
      <div className="diff-view">
        {fileGroups.map(({ filePath, hunks: fileHunks }) => {
          const allCosmetic =
            collapseCosmetic && hasNonSubstantive && fileHunks.every((h) => nonSubstantiveHunkIds.has(h.id));
          const collapsed = collapsedFiles.has(filePath) || allCosmetic;
          const adds = fileHunks.reduce((n, h) => n + h.lines.filter((l) => l.kind === "add").length, 0);
          const dels = fileHunks.reduce((n, h) => n + h.lines.filter((l) => l.kind === "remove").length, 0);
          const cosmeticCount = hasNonSubstantive ? fileHunks.filter((h) => nonSubstantiveHunkIds.has(h.id)).length : 0;
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
                  {cosmeticCount > 0 && <span className="cosmetic-badge"> ({cosmeticCount} cosmetic)</span>}
                </span>
              </div>
              {!collapsed &&
                fileHunks.map((hunk) => {
                  const isCosmetic = nonSubstantiveHunkIds.has(hunk.id);
                  const cosmeticCollapsed = isCosmetic && collapseCosmetic;
                  return (
                    <div key={hunk.id} className="hunk-block" style={cosmeticCollapsed ? { opacity: 0.5 } : undefined}>
                      <div className="hunk-header">
                        <span className={`hunk-id${isCosmetic ? " hunk-id-cosmetic" : ""}`}>{hunk.id}</span>
                        <span className="hunk-range">{hunk.header}</span>
                        {cosmeticCollapsed && <span className="cosmetic-badge">cosmetic</span>}
                      </div>
                      {!cosmeticCollapsed && (
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
                      )}
                    </div>
                  );
                })}
            </div>
          );
        })}
        {hunks.length === 0 && <p className="hint">No diff loaded.</p>}
      </div>
    </div>
  );
}
