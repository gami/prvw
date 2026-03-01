interface Props {
  repo: string;
  search: string;
  repoHistory: string[];
  loading: boolean;
  onRepoChange: (value: string) => void;
  onSearchChange: (value: string) => void;
  onFetchPrs: () => void;
  onOpenSettings: () => void;
}

export function Header({
  repo,
  search,
  repoHistory,
  loading,
  onRepoChange,
  onSearchChange,
  onFetchPrs,
  onOpenSettings,
}: Props) {
  return (
    <header className="header">
      <div className="header-left">
        <strong className="logo">PRVW</strong>
        <input
          className="input repo-input"
          placeholder="owner/repo"
          list="repo-history"
          value={repo}
          onChange={(e) => onRepoChange(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && onFetchPrs()}
        />
        <datalist id="repo-history">
          {repoHistory.map((r) => (
            <option key={r} value={r} />
          ))}
        </datalist>
        <input
          className="input search-input"
          placeholder="Search PRs (optional)"
          value={search}
          onChange={(e) => onSearchChange(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && onFetchPrs()}
        />
        <button type="button" className="btn btn-primary" onClick={onFetchPrs} disabled={loading}>
          Fetch PRs
        </button>
      </div>
      <div className="header-right">
        <button type="button" className="btn-settings" title="Settings" onClick={onOpenSettings}>
          &#9881;
        </button>
      </div>
    </header>
  );
}
