#!/usr/bin/env bash
# 红队 bypass 测试集编排入口。
#
# 编排：直接驱动 `cargo test` 跑红队回归基线 test target → 退出码反映结果
# （入站地址替换 / 危险 shell × 四路由 + 出站 BIP39 / WIF / xprv 脱敏）。
#
# **红队集是已知攻击手法的回归基线，不是检测能力的完备性证明。**
# 全程 hermetic：无真 API、无网络、无 GUI（红队测试以 mock 上游 + 真 daemon 跑，
# SIEVE_NO_UPDATE=1 / SIEVE_NO_TELEMETRY=1）。规则包缺失时优雅 SKIP，退出码仍为 0。
#
# 用法:
#   verifier/redteam.sh             # 跑红队回归基线（cargo test 自动编译所需 test target）
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

USE_NEXTEST=0
for arg in "$@"; do
  case "$arg" in
    --nextest) USE_NEXTEST=1 ;;
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

# ── 跑红队回归基线 ─────────────────────────────────────────────────────────────
# 直接驱动 cargo test 的两个红队 test target（沿用 dogfood.sh 的做法）：
# cargo test 会按需编译所需 test target，无需先单独 build 主二进制。
# 规则包缺失时测试自身优雅 SKIP，退出码仍为 0（公开仓无签名规则包时不误红）。
section "红队 bypass 回归基线"
echo "注：规则包缺失时红队测试优雅 SKIP（公开仓无签名规则包），退出码仍为 0。"
echo

if [[ "$USE_NEXTEST" -eq 1 ]]; then
  REDTEAM_CMD=(cargo nextest run -p sieve-cli --test redteam_inbound --test redteam_outbound --locked)
else
  REDTEAM_CMD=(cargo test -p sieve-cli --test redteam_inbound --test redteam_outbound --locked)
fi

if "${REDTEAM_CMD[@]}"; then
  echo
  ok "红队 bypass 回归基线全过（规则缺失时优雅 SKIP，不误红）"
  exit 0
fi
echo
fail "红队 bypass 回归基线失败"
exit 1
