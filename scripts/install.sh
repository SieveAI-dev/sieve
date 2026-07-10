#!/usr/bin/env bash
#
# Sieve 自校验安装器 — 一行安装 sieve CLI/daemon 二进制。
#
#   curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/SieveAI-dev/sieve/main/scripts/install.sh | bash
#
# 与绝大多数 `curl | sh` 不同：本脚本在把任何二进制落地到你机器之前，**先校验它**。
# Sieve 的 release 二进制由 GitHub Actions 用 cosign keyless（sigstore）签名，签名连同
# 证书与 Rekor 透明日志条目一起发布为 `<artifact>.sigstore.json` bundle。本脚本：
#
#   1. 若本机有 cosign         → 用 sigstore bundle 做密码学验签（最强：可追溯、防静默篡改）。
#   2. 否则                    → 回退到对照 SHA256SUMS 的 sha256 校验，并明确警告其局限，
#                                引导你装 cosign 做完整验证。
#   3. 任一校验失败            → **立即退出、不安装（fail-closed）**。
#
# 一个天天劝人"别把远程脚本盲管进 shell"的安全工具，自己的安装器理应长这样：
# 一行命令，照样可验。校验是安装器替你做的家庭作业，不是甩给你的门槛。
#
# 这只装 daemon/CLI 二进制（sieve）。GUI 走签名 .dmg 或 `brew install --cask sieve`。
#
# 可用环境变量覆盖：
#   SIEVE_INSTALL_DIR   安装目录（默认 ~/.local/bin）
#   SIEVE_VERSION       release tag（默认 latest）
#   SIEVE_REPO          GitHub owner/repo（默认 SieveAI-dev/sieve）
#
set -euo pipefail

# ── 配置 ────────────────────────────────────────────────────────────────────
SIEVE_REPO="${SIEVE_REPO:-SieveAI-dev/sieve}"
SIEVE_VERSION="${SIEVE_VERSION:-latest}"
SIEVE_INSTALL_DIR="${SIEVE_INSTALL_DIR:-$HOME/.local/bin}"
BIN_NAME="sieve"
# cosign 验签锚点：签名身份必须是本仓库 release workflow 在某个 vX.Y.Z tag 上的运行。
CERT_IDENTITY_REGEXP="^https://github.com/${SIEVE_REPO}/\.github/workflows/release\.yml@refs/tags/v[0-9.]+\$"
CERT_OIDC_ISSUER="https://token.actions.githubusercontent.com"

# ── 输出辅助 ────────────────────────────────────────────────────────────────
if [ -t 2 ]; then
  c_red=$'\033[31m'; c_grn=$'\033[32m'; c_ylw=$'\033[33m'; c_dim=$'\033[2m'; c_rst=$'\033[0m'
else
  c_red=''; c_grn=''; c_ylw=''; c_dim=''; c_rst=''
fi
info()  { printf '%s\n' "${c_dim}sieve${c_rst} $*" >&2; }
ok()    { printf '%s\n' "${c_grn}✓${c_rst} $*" >&2; }
warn()  { printf '%s\n' "${c_ylw}!${c_rst} $*" >&2; }
die()   { printf '%s\n' "${c_red}✗ $*${c_rst}" >&2; exit 1; }

# ── 临时目录（退出时清理）───────────────────────────────────────────────────
TMPDIR_DL="$(mktemp -d "${TMPDIR:-/tmp}/sieve-install.XXXXXX")"
cleanup() { rm -rf "$TMPDIR_DL"; }
trap cleanup EXIT

# ── 平台检测（macOS 工作；Linux 预留位）─────────────────────────────────────
detect_artifact() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"
  case "$os" in
    Darwin)
      # release 产出 universal 二进制（arm64 + x86_64 合并），单架构机器也能跑。
      echo "${BIN_NAME}-macos-universal"
      ;;
    Linux)
      # 预留位：Tier 1 Linux musl 目标（供应链硬约束，.cursorrules §二 #6）尚未在 release 产出对应二进制。
      # 待 release.yml 恢复 Linux matrix 后，按 arch 映射 sieve-<arch>-unknown-linux-musl。
      die "Linux 安装器尚未就绪（pre-GA 仅 macOS）。请从源码构建：cargo install --git https://github.com/${SIEVE_REPO} sieve-cli"
      ;;
    *)
      die "不支持的平台：$os $arch"
      ;;
  esac
}

# ── 依赖检查 ────────────────────────────────────────────────────────────────
need() { command -v "$1" >/dev/null 2>&1; }
sha256_tool() {
  if need shasum; then echo "shasum -a 256";
  elif need sha256sum; then echo "sha256sum";
  else die "未找到 shasum / sha256sum，无法做 sha256 校验"; fi
}

# ── 下载（curl 硬化：仅 https + TLS≥1.2 + 失败即报错）────────────────────────
fetch() {
  local url="$1" out="$2"
  curl --proto '=https' --tlsv1.2 -fsSL "$url" -o "$out" \
    || die "下载失败：$url"
}

# ── 校验：cosign 优先，sha256 兜底，fail-closed ─────────────────────────────
verify() {
  local artifact="$1" bin="$2" bundle="$3" sums="$4"

  if need cosign; then
    info "用 cosign 验签（sigstore keyless + Rekor 透明日志）…"
    if cosign verify-blob \
        --certificate-identity-regexp "$CERT_IDENTITY_REGEXP" \
        --certificate-oidc-issuer "$CERT_OIDC_ISSUER" \
        --bundle "$bundle" \
        "$bin" >/dev/null 2>&1; then
      ok "cosign 验签通过：签名来自 ${SIEVE_REPO} 的 release workflow，且已记入 Rekor。"
      return 0
    fi
    die "cosign 验签未通过 —— 二进制可能被篡改或来源不符。已中止安装（fail-closed）。"
  fi

  # 回退：sha256 校验。仅能防传输损坏/部分篡改，无法证明来源真实性（SHA256SUMS 本身未签名）。
  warn "未找到 cosign，回退到 sha256 校验。"
  warn "sha256 仅防传输损坏，${c_red}不${c_rst}${c_ylw}提供来源真实性保证${c_rst}（SHA256SUMS 文件本身无签名）。"
  warn "完整验证请装 cosign 后重跑：${c_dim}brew install cosign${c_rst}"
  local expected actual stool
  stool="$(sha256_tool)"
  expected="$(grep -E "  ${artifact}\$" "$sums" | awk '{print $1}' | head -n1)"
  [ -n "$expected" ] || die "SHA256SUMS 中找不到 ${artifact} 的校验和。已中止（fail-closed）。"
  actual="$($stool "$bin" | awk '{print $1}')"
  if [ "$expected" = "$actual" ]; then
    ok "sha256 匹配：$actual"
    return 0
  fi
  die "sha256 不匹配（期望 $expected，实际 $actual）。二进制可能损坏或被篡改。已中止（fail-closed）。"
}

# ── 安装 ────────────────────────────────────────────────────────────────────
main() {
  need curl || die "未找到 curl。"
  local artifact base bin bundle sums
  artifact="$(detect_artifact)"
  if [ "$SIEVE_VERSION" = "latest" ]; then
    base="https://github.com/${SIEVE_REPO}/releases/latest/download"
  else
    base="https://github.com/${SIEVE_REPO}/releases/download/${SIEVE_VERSION}"
  fi

  info "目标：${artifact}（来自 ${SIEVE_REPO} ${SIEVE_VERSION}）"
  bin="$TMPDIR_DL/$artifact"
  bundle="$TMPDIR_DL/${artifact}.sigstore.json"
  sums="$TMPDIR_DL/SHA256SUMS"

  info "下载二进制 + 签名 bundle…"
  fetch "$base/$artifact" "$bin"
  fetch "$base/${artifact}.sigstore.json" "$bundle"
  fetch "$base/SHA256SUMS" "$sums"

  verify "$artifact" "$bin" "$bundle" "$sums"

  # 校验通过后才落地（fail-closed：上面任一 die 都不会走到这里）。
  mkdir -p "$SIEVE_INSTALL_DIR"
  install -m 0755 "$bin" "$SIEVE_INSTALL_DIR/$BIN_NAME"
  ok "已安装：$SIEVE_INSTALL_DIR/$BIN_NAME"

  # PATH 提示。
  case ":$PATH:" in
    *":$SIEVE_INSTALL_DIR:"*) : ;;
    *) warn "$SIEVE_INSTALL_DIR 不在 PATH 中。把这行加进你的 shell 配置（~/.zshrc 等）：";
       printf '%s\n' "    export PATH=\"$SIEVE_INSTALL_DIR:\$PATH\"" >&2 ;;
  esac

  printf '\n' >&2
  ok "安装完成。下一步："
  printf '%s\n' "    ${c_dim}sieve setup${c_rst}   # 注册 Claude Code hook + 设 ANTHROPIC_BASE_URL + 装 launchd" >&2
  printf '%s\n' "    ${c_dim}sieve doctor${c_rst}  # 体检（含安装来源/验证状态）" >&2
}

main "$@"
