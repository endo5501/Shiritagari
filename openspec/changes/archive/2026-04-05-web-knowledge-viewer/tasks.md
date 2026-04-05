## 1. 依存追加とモジュール構造

- [x] 1.1 Cargo.toml に axum, askama, askama_axum の依存を追加
- [x] 1.2 `src-tauri/src/web/mod.rs` と `src-tauri/src/web/handlers.rs` を作成し、lib.rs に `mod web` を追加

## 2. DB層のページング対応

- [x] 2.1 `memory/patterns.rs` に `count_active_patterns()` と `get_active_patterns_paginated(limit, offset)` を追加し、テストを作成
- [x] 2.2 `memory/episodes.rs` に `count_episodes()` と `get_episodes_paginated(limit, offset)` を追加し、テストを作成

## 3. askama テンプレート

- [x] 3.1 `src-tauri/templates/base.html` を作成（共通レイアウト、ナビゲーション、CSS）
- [x] 3.2 `src-tauri/templates/patterns.html` を作成（パターン一覧 + ページング）
- [x] 3.3 `src-tauri/templates/episodes.html` を作成（エピソード一覧 + ページング）
- [x] 3.4 `src-tauri/templates/profile.html` を作成（プロフィール表示）

## 4. Web サーバ実装

- [x] 4.1 `web/mod.rs` にポート探索ロジックと axum Router 構築・サーバ起動関数を実装
- [x] 4.2 `web/handlers.rs` に GET `/` (Patterns)、`/episodes`、`/profile` のハンドラを実装

## 5. Tauri 統合

- [x] 5.1 `lib.rs` の setup 内で HTTP サーバを `tokio::spawn` で起動し、ポートを `AppState` に保持
- [x] 5.2 トレイメニューに「Knowledge Base」項目を追加し、クリックでブラウザを開く処理を実装

## 6. テスト

- [x] 6.1 Web ハンドラの統合テスト（axum::test を使用してレスポンスを検証）
