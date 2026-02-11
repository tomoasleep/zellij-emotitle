# zellij-emotitle

Zellij の pane / tab タイトルに `(<original title>) | (<emojis>)` の形式で emoji を付与するプラグインです。

- 付与は `zellij pipe` で実行
- temporary (`mode=temp`) は対象がフォーカスされたタイミングで元に戻す
- permanent (`mode=permanent`) は維持
- `pane_id` から `tab_index` を解決して tab に付与可能

## ビルド

```bash
rustup target add wasm32-wasip1
cargo build --release --target wasm32-wasip1
```

生成物:

`target/wasm32-wasip1/release/zellij_emotitle.wasm`

## 読み込み

`~/.config/zellij/config.kdl` などで background ロードします。

```kdl
load_plugins {
  file:/ABSOLUTE/PATH/TO/zellij_emotitle.wasm
}
```

または `zellij pipe --plugin file:/.../zellij_emotitle.wasm` で初回メッセージ時に自動起動できます。

## 引数形式

`zellij pipe` の `--args` を使って指定します。

- `target`: `pane` または `tab` (必須)
- `emojis`: 付与する絵文字 (必須)
- `mode`: `temp` or `permanent` (省略時 `temp`)
- `pane_id`: pane id (任意)
- `tab_index`: tab index (0-based, 任意)

`target=tab` のとき `pane_id` と `tab_index` は同時指定できません。

## 使い方

### 1) フォーカス中の pane に付与

```bash
zellij pipe \
  --name emotitle \
  --plugin file:/ABSOLUTE/PATH/TO/zellij_emotitle.wasm \
  --args target=pane,emojis=🚀,mode=temp
```

### 2) 指定 pane_id に付与

```bash
zellij pipe \
  --name emotitle \
  --plugin file:/ABSOLUTE/PATH/TO/zellij_emotitle.wasm \
  --args target=pane,pane_id=12,emojis=✅,mode=permanent
```

### 3) フォーカス中の tab に付与

```bash
zellij pipe \
  --name emotitle \
  --plugin file:/ABSOLUTE/PATH/TO/zellij_emotitle.wasm \
  --args target=tab,emojis=📚,mode=temp
```

### 4) pane_id から tab を解決して付与

```bash
zellij pipe \
  --name emotitle \
  --plugin file:/ABSOLUTE/PATH/TO/zellij_emotitle.wasm \
  --args target=tab,pane_id=12,emojis=🔥,mode=permanent
```

## ZELLIJ_PANE_ID / ZELLIJ_SESSION_NAME だけで tab を特定する

外部スクリプトからは以下の形で利用できます。

```bash
zellij --session "$ZELLIJ_SESSION_NAME" pipe \
  --name emotitle \
  --plugin file:/ABSOLUTE/PATH/TO/zellij_emotitle.wasm \
  --args target=tab,pane_id=$ZELLIJ_PANE_ID,emojis=🔔,mode=temp
```

このときプラグイン側で `pane_id -> tab_index` を `PaneUpdate` 情報から解決します。

## 返り値

`zellij pipe` の stdout に `ok` またはエラーメッセージを返します。
