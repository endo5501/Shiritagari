## Why

AIが学習した知識（パターン、エピソード、プロフィール）はSQLiteに保存されているが、内容を確認するにはDBを直接開く必要がある。ユーザが「AIが何を覚えているか」を手軽に確認できるWebインタフェースを提供し、透明性と信頼感を高める。

## What Changes

- Rust側に組み込みHTTPサーバ（axum）を追加し、localhostでWeb UIを提供
- askama テンプレートエンジンでHTML を生成（読み取り専用）
- Patterns 一覧、Episodes 一覧（ページング付き）、User Profile の3ページ
- システムトレイメニューに「Knowledge Base」項目を追加し、クリックでデフォルトブラウザを起動
- ポート 14789 を基点に空きポートを自動探索

## Capabilities

### New Capabilities
- `web-knowledge-viewer`: 組み込みHTTPサーバによるナレッジベースのWeb閲覧機能（Patterns、Episodes、Profile の読み取り専用表示、ページング）

### Modified Capabilities
- `memory-store`: ページング対応のためにクエリメソッドに limit/offset パラメータを追加

## Impact

- **依存追加**: `axum`, `tower-http`, `askama`, `askama_axum` を Cargo.toml に追加
- **Rust コード**: `src-tauri/src/web/` モジュール新設、`lib.rs` にサーバ起動とトレイメニュー拡張
- **テンプレート**: `src-tauri/templates/` にHTML テンプレートファイルを追加
- **DB層**: `memory/patterns.rs`, `memory/episodes.rs` に count / paginated query メソッド追加
