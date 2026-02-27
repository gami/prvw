import { useState } from "react";

const REPO_HISTORY_KEY = "prvw:repoHistory";
const REPO_HISTORY_MAX = 20;

function loadHistory(): string[] {
  try {
    const raw = localStorage.getItem(REPO_HISTORY_KEY);
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

function saveToHistory(repo: string) {
  const history = loadHistory().filter((r) => r !== repo);
  history.unshift(repo);
  localStorage.setItem(REPO_HISTORY_KEY, JSON.stringify(history.slice(0, REPO_HISTORY_MAX)));
}

export function useRepoHistory() {
  const [repo, setRepo] = useState(() => localStorage.getItem("prvw:repo") ?? "");
  const [repoHistory, setRepoHistory] = useState(loadHistory);

  function persistRepo(repoValue: string) {
    localStorage.setItem("prvw:repo", repoValue);
    saveToHistory(repoValue);
    setRepoHistory(loadHistory());
  }

  return { repo, setRepo, repoHistory, persistRepo };
}
