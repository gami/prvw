# PRVW — PR Review Viewer

GitHub PR の diff を取得し、構造化し、LLMで修正意図ごとにグルーピングしてレビューしやすくするためのdiffビューワ。

## スタック

| ツール | 用途 | インストール |
|--------|------|-------------|
| **Node.js** (v18+) | フロントビルド | https://nodejs.org/ |
| **Rust** (stable) | バックエンド | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **GitHub CLI (`gh`)** | PR一覧・diff取得 | `brew install gh` |
| **Codex CLI (`codex`)** | Intent分析（任意） | https://github.com/openai/codex |

## セットアップ

```bash
# 1. 依存インストール
npm install

# 2. gh の認証
gh auth login

# 3. Codex の認証（Intent分析を使う場合）
codex login
```

## 開発サーバー起動

```bash
npm run tauri dev
```

## 使い方

1. ヘッダーの入力欄にリポジトリ名（例: `facebook/react`）を入力し「Fetch PRs」をクリック
2. PR一覧が表示されるのでクリックして選択 → diff が取得・パースされ中央ペインに表示
3. 左ペインの「Run Codex Analysis」をクリック → Codex が hunk を修正意図ごとにグルーピング
4. グループをクリックすると中央ペインがそのグループの hunk だけに絞り込まれ、右ペインにサマリーが表示
5. チェックボックスでレビュー済みマーク（セッション中のみ）

## トラブルシューティング

| 症状 | 対処 |
|------|------|
| `gh is not installed` | `brew install gh` でインストール |
| `gh is not authenticated` | `gh auth login` を実行 |
| `Codex CLI is not installed` | Codex をインストール（Intent分析なしでも diff 表示は可能） |
| `Codex CLI is not authenticated` | `codex login` を実行 |
| diff が空 | PR に変更がない場合。ブランチ比較を確認 |
| Analysis validation error | Codex の出力が不正。再実行で解決することが多い |

## プロジェクト構成

```
prvw/
├── src/                    # フロントエンド (React + TS)
│   ├── App.tsx             # メインUI（3ペインレイアウト）
│   ├── App.css             # スタイル
│   ├── types.ts            # TypeScript型定義
│   └── main.tsx            # エントリーポイント
├── src-tauri/              # バックエンド (Rust)
│   └── src/
│       ├── lib.rs          # Tauriアプリ初期化
│       └── commands.rs     # コマンド実装（gh/codex呼び出し、diffパース）
├── index.html
└── package.json
```
