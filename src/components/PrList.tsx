import { useEffect, useRef } from "react";
import type { PrListItem } from "../types";

interface Props {
  prs: PrListItem[];
  onSelect: (pr: PrListItem) => void;
  onLoadMore: () => void;
  hasMore: boolean;
  loadingMore: boolean;
}

export function PrList({ prs, onSelect, onLoadMore, hasMore, loadingMore }: Props) {
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = scrollRef.current;
    if (!el) return;
    function handleScroll() {
      if (!el || !hasMore || loadingMore) return;
      if (el.scrollTop + el.clientHeight >= el.scrollHeight - 50) {
        onLoadMore();
      }
    }
    el.addEventListener("scroll", handleScroll, { passive: true });
    handleScroll();
    return () => el.removeEventListener("scroll", handleScroll);
  }, [hasMore, loadingMore, onLoadMore]);

  return (
    <div className="pr-list" ref={scrollRef}>
      <table>
        <thead>
          <tr>
            <th>#</th>
            <th>Title</th>
            <th>Author</th>
            <th>Branch</th>
            <th>Updated</th>
          </tr>
        </thead>
        <tbody>
          {prs.map((pr) => (
            <tr key={pr.number} onClick={() => onSelect(pr)} className="pr-row">
              <td>{pr.number}</td>
              <td>
                {pr.title}
                {pr.isDraft ? (
                  <span className="pr-badge pr-badge-draft">DRAFT</span>
                ) : pr.reviewDecision === "APPROVED" ? (
                  <span className="pr-badge pr-badge-approved">APPROVED</span>
                ) : pr.reviewDecision === "CHANGES_REQUESTED" ? (
                  <span className="pr-badge pr-badge-changes">CHANGES REQUESTED</span>
                ) : pr.reviewDecision === "REVIEW_REQUIRED" ? (
                  <span className="pr-badge pr-badge-review">REVIEW REQUIRED</span>
                ) : null}
              </td>
              <td>{pr.author?.login ?? "—"}</td>
              <td className="branch-cell">{pr.headRefName && <span className="branch">{pr.headRefName}</span>}</td>
              <td>{pr.updatedAt ? new Date(pr.updatedAt).toLocaleDateString() : "—"}</td>
            </tr>
          ))}
        </tbody>
      </table>
      {loadingMore && <div className="pr-list-loading-more">Loading more PRs...</div>}
      {!hasMore && prs.length > 0 && <div className="pr-list-end">All PRs loaded</div>}
    </div>
  );
}
