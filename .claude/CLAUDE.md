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

## change作成時の注意

OpenSpecのスキルでchange作成した際、同時に開発用ブランチを作成してください

## tasks.md作成時の注意

OpenSpecのスキルでtasks.mdを作成する際、最終確認のため以下の項目を追加してください

```md
## X. 最終確認

- [ ] X.1 `/simplify`スキルを使用してコードレビューを実施
- [ ] X.2 `/codex:review --scope branch --background` スキルを使用して現在開発中のコードレビューを実施
- [ ] X.3 `/opsx:verify`でcahngeを検証
```

## archive時の注意

必ずDelta specの同期を行なってください
