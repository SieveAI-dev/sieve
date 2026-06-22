#!/usr/bin/env bash
# ADR-043 红队 bypass 测试集编排入口。
#
# 编排：构建 sieve 二进制 → 调 `sieve verify redteam` headless 驱动红队回归基线
# （入站地址替换 / 危险 shell × 四路由 + 出站 BIP39 / WIF / xprv 脱敏）→ 退出码反映结果。
#
# **红队集是已知攻击手法的回归基线，不是检测能力的完备性证明。**
# 全程 hermetic：无真 API、无网络、无 GUI（红队测试以 mock 上游 + 真 daemon 跑，
# SIEVE_NO_UPDATE=1 / SIEVE_NO_TELEMETRY=1）。规则包缺失时优雅 SKIP，退出码仍为 0。
#
# 用法:
#   verifier/redteam.sh             # 构建 + 跑红队回归基线
#   verifier/redteam.sh --fast      # 跳过构建（用已存在的二进制）
#   verifier/redteam.sh --no-build  # 同 --fast
#   verifier/redteam.sh --nextest   # 用 cargo nextest run 代替 cargo test（CI profile）
#
# 退出码: 0 全过（含 SKIP）/ 非 0 有失败。CI 直接用退出码判定。
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# cargo 不一定在默认 PATH（rustup 装在 ~/.cargo/bin，shim 未必进 PATH）。
if ! command -v cargo >/dev/null 2>&1; then
  if [[ -x "$HOME/.cargo/bin/cargo" ]]; then
    export PATH="$HOME/.cargo/bin:$PATH"
  else
    echo "✗ 找不到 cargo（既不在 PATH 也不在 ~/.cargo/bin）" >&2
    exit 2
  fi
fi

DO_BUILD=1
NEXTEST_FLAG=""
for arg in "$@"; do
  case "$arg" in
    --fast | --no-build) DO_BUILD=0 ;;
    --nextest) NEXTEST_FLAG="--nextest" ;;
    *) echo "未知参数: $arg" >&2; exit 2 ;;
  esac
done

# 标记色（CI 无 tty 时降级为纯文本）。
if [[ -t 1 ]]; then
  BOLD=$'\033[1m'; GREEN=$'\033[32m'; RED=$'\033[31m'; RST=$'\033[0m'
else
  BOLD=""; GREEN=""; RED=""; RST=""
fi
section() { echo; echo "${BOLD}=== $* ===${RST}"; }
ok() { echo "${GREEN}✓ $*${RST}"; }
fail() { echo "${RED}✗ $*${RST}"; }

# ── 1. 构建 sieve 二进制（verify redteam 子命令所需）─────────────────────────────
if [[ "$DO_BUILD" -eq 1 ]]; then
  section "1. 构建 sieve 二进制"
  if cargo build -p sieve-cli --locked; then
    ok "sieve 二进制就绪"
  else
    fail "构建失败"
    exit 1
  fi
else
  section "1. 跳过构建（--fast）"
  if [[ ! -x target/release/sieve && ! -x target/debug/sieve ]]; then
    fail "无现成二进制，--fast 不可用（先去掉 --fast 跑一次构建）"
    exit 1
  fi
  ok "复用已存在二进制"
fi

# ── 2. 跑红队回归基线（sieve verify redteam）──────────────────────────────────
section "2. 红队 bypass 回归基线 (ADR-043)"
SIEVE_BIN="target/debug/sieve"
[[ -x target/release/sieve ]] && SIEVE_BIN="target/release/sieve"

if "$SIEVE_BIN" verify redteam ${NEXTEST_FLAG:+$NEXTEST_FLAG}; then
  echo
  ok "红队 bypass 回归基线全过（规则缺失时优雅 SKIP，不误红）"
  exit 0
fi
echo
fail "红队 bypass 回归基线失败"
exit 1
