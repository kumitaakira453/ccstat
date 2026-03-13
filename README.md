# ccstat

Claude Codeのセッションログを解析し、プロジェクトごとの行数統計を表示するCLIツール。

## インストール

### Homebrew

```bash
brew tap kumitaakira453/tap
brew install ccstat
```

### cargo install

```bash
cargo install --git https://github.com/kumitaakira453/ccstat
```

### ソースからビルド

```bash
git clone https://github.com/kumitaakira453/ccstat
cd ccstat
cargo build --release
# バイナリ: target/release/ccstat
```

## 使い方

```bash
# プロジェクトごとのサマリー
ccstat

# セッション単位の内訳
ccstat --session

# 特定プロジェクトのみ表示
ccstat --project my-project

# カスタムログディレクトリを指定
ccstat --path /path/to/.claude/projects
```

## 出力例

### サマリー表示

| プロジェクト |     Write |      Edit |  チャット |  コード計 |    総合計 |
| ------------ | --------: | --------: | --------: | --------: | --------: |
| project-a    |     1,245 |       832 |     3,567 |     2,077 |     5,644 |
| project-b    |       456 |       234 |     1,890 |       690 |     2,580 |
| **合計**     | **1,701** | **1,066** | **5,457** | **2,767** | **8,224** |

### セッション表示 (`--session`)

| 日付       | タイトル             |     Write |    Edit |  チャット |      合計 |
| ---------- | -------------------- | --------: | ------: | --------: | --------: |
| 2025-06-03 | Add user auth        |       325 |     200 |     1,200 |     1,725 |
| 2025-06-02 | Fix login bug        |       500 |     322 |     1,167 |     1,989 |
| 2025-06-01 | Refactor API hand... |       420 |     310 |     1,200 |     1,930 |
|            | **合計**             | **1,245** | **832** | **3,567** | **5,644** |

## 解析対象

`~/.claude/projects/` 配下のJSONLログファイルを解析します。

| カテゴリ | 内容                                       |
| -------- | ------------------------------------------ |
| Write    | `tool_use` で `Write` ツールが使われた行数 |
| Edit     | `tool_use` で `Edit` ツールが使われた行数  |
| チャット | アシスタントのテキスト応答の行数           |
| コード計 | Write + Edit                               |
| 総合計   | Write + Edit + チャット                    |

## ライセンス

MIT
