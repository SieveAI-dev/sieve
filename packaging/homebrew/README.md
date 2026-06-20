# Homebrew packaging

本目录存放 Sieve 的 Homebrew formula 与 cask。Homebrew tap 通常是**独立仓库**，所以这里只是
源副本 —— 发布时要把它们复制到 tap 仓库 `SieveAI-dev/homebrew-sieve`。

| 文件 | 装什么 | 用户命令 |
|------|--------|---------|
| `sieve.rb`（formula） | CLI/daemon 二进制 `sieve` | `brew install sieve` |
| `Casks/sieve.rb`（cask） | GUI `SieveGUI.app` | `brew install --cask sieve` |

## 为什么用 Homebrew

brew 用 formula/cask 内的 `sha256` **自动校验**下载产物——校验是包管理器原生提供的，
用户无需手动验签。这与 Sieve 的 `scripts/install.sh`（cosign 自校验）一脉相承：
**安装方式无摩擦，但产物照样被校验，校验不过就 fail-closed。**

## tap 仓库结构

```
homebrew-sieve/
├── Formula/
│   └── sieve.rb      ← 复制自 packaging/homebrew/sieve.rb
└── Casks/
    └── sieve.rb      ← 复制自 packaging/homebrew/Casks/sieve.rb
```

用户首次使用：

```bash
brew tap SieveAI-dev/sieve
brew install sieve              # CLI
brew install --cask sieve       # GUI
```

## 发版流程（每个 release 重复）

formula/cask 当前的 `version` 与 `sha256` 是 **pre-GA 占位**（全零 sha256 会让 brew
校验失败，这是有意的 fail-closed）。首个 release 发布后：

1. **CLI（`sieve.rb`）**：从 sieve 仓 release 的 `SHA256SUMS` 取 `sieve-aarch64-apple-darwin`
   与 `sieve-x86_64-apple-darwin` 的真实 sha256，填入对应 `on_arm`/`on_intel` 块；把 `version`
   改为该 release tag（去掉 `v` 前缀）。
2. **GUI（`Casks/sieve.rb`）**：从 sieve-gui-macos 仓 release 取 `.dmg` 的 sha256
   （`shasum -a 256 SieveGUI-<ver>.dmg`），填入 `sha256`；改 `version`。
3. 把更新后的两个文件复制到 `homebrew-sieve` 仓并提交。
4. 验证：`brew install --build-from-source ./sieve.rb` 或 `brew audit --strict sieve`。

> 自动化建议（Phase 2）：在 sieve / sieve-gui-macos 的 release workflow 末尾加一步，
> 自动用真实 sha256 渲染这两个文件并向 `homebrew-sieve` 提 PR。
