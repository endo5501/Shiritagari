# Shiritagari

ユーザのPC操作を能動的に観察・質問・学習する常駐型AIエージェント。

[ActivityWatch](https://activitywatch.net/)と連携してウィンドウ操作を定期的にモニタリングし、不明な行動についてLLMで推論・質問し、回答を記憶して学習するループを実現します。

## 開発コマンド

* `npm test`                  # フロントエンドテスト（vitest）
* `npm run test:rust`         # Rustユニットテスト
* `npm run test:typecheck`    # TypeScript型チェック
* `npm run test:watch`        # フロントエンドテスト（watchモード）
* `npm run test:all`          # 全テスト一括実行（型チェック + フロントエンド + Rust）
* `npm run tauri build`       # プロダクションビルド（OS別のバイナリを生成）
* 