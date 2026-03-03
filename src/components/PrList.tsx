import type { PrListItem } from "../types";

interface Props {
  prs: PrListItem[];
  onSelect: (pr: PrListItem) => void;
}

export function PrList({ prs, onSelect }: Props) {
  return (
    <div className="pr-list">
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
              <td>{pr.title}</td>
              <td>{pr.author?.login ?? "—"}</td>
              <td className="branch-cell">{pr.headRefName && <span className="branch">{pr.headRefName}</span>}</td>
              <td>{pr.updatedAt ? new Date(pr.updatedAt).toLocaleDateString() : "—"}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
