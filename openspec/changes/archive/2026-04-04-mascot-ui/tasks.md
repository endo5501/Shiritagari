## 1. Tauri ウィンドウ設定変更

- [x] 1.1 `tauri.conf.json` を変更: `transparent: true`, `decorations: false`, ウィンドウサイズを 300x500 に調整
- [x] 1.2 `index.html` の `body`/`html` 背景を透明に設定
- [x] 1.3 透過ウィンドウの動作確認（macOS）

## 2. 設定ファイルへのマスコット設定追加

- [x] 2.1 `config.rs` に `MascotConfig` 構造体を追加（`character_image` パスフィールド）
- [x] 2.2 `AppConfig` に `mascot` セクションを追加
- [x] 2.3 設定読み込みのテストを追加

## 3. バックエンドイベント送信の追加

- [x] 3.1 `lib.rs` の推論ループに `shiritagari-thought` イベント送信を追加（LLM推論完了時に inference と confidence を送信）
- [x] 3.2 質問発生時に `set_always_on_top(true)` を呼び出す処理を追加
- [x] 3.3 `answer_question` コマンド内で `set_always_on_top(false)` に戻す処理を追加

## 4. LLMプロンプト修正

- [x] 4.1 `providers/claude.rs` の `build_inference_prompt` で inference フィールドの説明を「短く独り言風に、40文字以内」に変更
- [x] 4.2 `providers/openai.rs` に同じプロンプト変更を適用
- [x] 4.3 `providers/ollama.rs` に同じプロンプト変更を適用
- [x] 4.4 プロンプト変更後の推論出力形式テスト

## 5. フロントエンド: マスコットUI実装

- [x] 5.1 `App.tsx` を全面書き換え: チャットUIからマスコットUIへ（キャラ画像表示、吹き出し、入力欄）
- [x] 5.2 キャラクター画像コンポーネント: `convertFileSrc` でローカルPNG読み込み、`data-tauri-drag-region` 属性付与
- [x] 5.3 `shiritagari-thought` イベントリスナーを追加し、思考テキストを状態管理
- [x] 5.4 `shiritagari-question` イベント受信時に発話モードに切り替え
- [x] 5.5 回答送信後に思考モードに復帰する処理

## 6. フロントエンド: 吹き出しスタイル

- [x] 6.1 `App.css` を全面書き換え: 透過背景、マスコットレイアウト
- [x] 6.2 思考吹き出しスタイル: 丸角、○尻尾、半透明背景
- [x] 6.3 発話吹き出しスタイル: 角丸四角、╲尻尾、不透明背景・目立つ色調
- [x] 6.4 入力欄スタイル: 不透明背景で可読性確保、キャラクター下部に配置

## 7. デフォルトアセット

- [x] 7.1 デフォルトのプレースホルダーキャラクター画像を作成・配置
- [x] 7.2 アセットの読み込みパス設定

## 8. テストと動作確認

- [x] 8.1 既存のフロントエンドテストをマスコットUIに合わせて更新
- [x] 8.2 Rustユニットテスト: 設定読み込み（MascotConfig）
- [x] 8.3 統合動作確認: 透過ウィンドウ + PNG表示 + 吹き出し表示 + ドラッグ移動
- [ ] 8.4 統合動作確認: 質問時の最前面浮上 + 回答後の復帰（別機会で確認）
