## MODIFIED Requirements

### Requirement: 透過ウィンドウによるマスコット表示
システムはタイトルバーなし・背景透過・シャドウ無効のウィンドウ上にキャラクターPNG画像を表示しなければならない（SHALL）。

#### Scenario: アプリ起動時のマスコット表示
- **WHEN** アプリが起動した時
- **THEN** 透過背景・装飾なし・シャドウ無効のウィンドウにキャラクターPNG画像を表示する

#### Scenario: 設定ファイルで指定されたPNGの読み込み
- **WHEN** config.tomlの `[mascot]` セクションに `character_image` パスが設定されている時
- **THEN** 指定パスの透過PNG画像をキャラクターとして表示する

#### Scenario: PNG未設定時のフォールバック
- **WHEN** `character_image` が未設定またはファイルが存在しない時
- **THEN** バンドル済みのデフォルトプレースホルダー画像を表示する
