#!/usr/bin/env bash
# Sieve dogfood 一键自动化入口。
#
# 完全 hermetic：无需真 ANTHROPIC_API_KEY、无需真网络、无需真 GUI。
# 全程本地 mock 上游 + headless CLI（sieve decisions / sieve audit）当决策客户端。
#
# 用法:
#   scripts/dogfood.sh            # 跑全套（构建 + cargo e2e + smoke + updater 闭环）
#   scripts/dogfood.sh --fast     # 跳过 release 构建（用已存在的二进制）
#   scripts/dogfood.sh --no-build # 同 --fast
#
# 退出码: 0 全过 / 非 0 有失败。CI 直接用退出码判定。
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
for arg in "$@"; do
  case "$arg" in
    --fast | --no-build) DO_BUILD=0 ;;
    *) echo "未知参数: $arg" >&2; exit 2 ;;
  esac
done

# 标记色（CI 无 tty 时降级为纯文本）。
if [[ -t 1 ]]; then
  BOLD=$'\033[1m'; GREEN=$'\033[32m'; RED=$'\033[31m'; DIM=$'\033[2m'; RST=$'\033[0m'
else
  BOLD=""; GREEN=""; RED=""; DIM=""; RST=""
fi

FAILED=0
section() { echo; echo "${BOLD}=== $* ===${RST}"; }
ok() { echo "${GREEN}✓ $*${RST}"; }
fail() { echo "${RED}✗ $*${RST}"; FAILED=1; }

# ── 1. 构建 release 二进制（harness 与 smoke 都需要真二进制）───────────────────
if [[ "$DO_BUILD" -eq 1 ]]; then
  section "1. 构建 release 二进制"
  if cargo build --release -p sieve-cli --locked; then
    ok "sieve 二进制就绪"
  else
    fail "构建失败"
    echo "${RED}构建失败，终止。${RST}" >&2
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

# ── 2. Hermetic cargo 集成测试（sieve-testing harness + e2e 场景）──────────────
# sieve-testing 的 self_test + dogfood_e2e（P1.2 落地后）全在这一步；全程 mock 上游。
section "2. Hermetic cargo e2e（mock 上游，无真 API/网络）"
if cargo test -p sieve-testing --locked; then
  ok "sieve-testing harness e2e 通过"
else
  fail "sieve-testing harness e2e 失败"
fi
# dogfood_e2e 集成测试（P1.2 落地后启用；现在没有则跳过不报错）。
if cargo test -p sieve-cli --test dogfood_e2e --locked 2>/dev/null; then
  ok "dogfood_e2e 场景通过"
else
  echo "${DIM}  (dogfood_e2e 暂未落地，跳过 —— P1.2)${RST}"
fi

# ── 3. Python smoke（黑盒：起 daemon + mock 上游跑透传/SSE/tool_use/脱敏）──────
section "3. Python smoke (--mock-only)"
if python3 scripts/smoke_test.py --mock-only; then
  ok "smoke --mock-only 通过"
else
  fail "smoke --mock-only 失败"
fi

# ── 4. updater 闭环（hermetic：plain-HTTP mock + SIEVE_CACHE_DIR 隔离，跨平台）──────
# §14.1 install-id / §14.4 fetch→download→sha256→zstd 解压→原子落盘 / §14.5 失败模式 /
# §14.6 公钥 None skip。无真网络/TLS/~/Library 依赖。
section "4. updater 闭环 (§14)"
if cargo test -p sieve-updater --test updater_e2e --locked; then
  ok "updater 闭环通过"
else
  fail "updater 闭环失败"
fi

# ── 5. FP/recall 数据集门（PRD §9 #7 Critical FP<0.5%；MVP G6 验证维度）──────────
# benign 1070 样本（含 900 个 near-miss 最严格场景）Critical FP 必须为 0；
# attack 数据集（含 public-replay 真实攻击复现）recall 必须 ≥99%。
# 此前 dataset_fp_rate 标 #[ignore] 从未进 CI（PoC/MVP 评估抓出的 gap）——
# 现纳入 dogfood 作回归门，防 Critical 误报回归（误报 >0.5% 触 PRD 公理 12，用户禁用产品）。
section "5. FP/recall 数据集门 (§9 #7)"
if cargo test -p sieve-rules --release --test dataset_fp_rate -- --ignored; then
  ok "Critical FP<0.5% + attack recall 门通过"
else
  fail "FP/recall 门失败（Critical 误报或召回退化）"
fi

# ── 总结 ──────────────────────────────────────────────────────────────────────
section "总结"
if [[ "$FAILED" -eq 0 ]]; then
  echo "${GREEN}${BOLD}dogfood 全部通过 ✓${RST}"
  exit 0
fi
echo "${RED}${BOLD}dogfood 有失败 ✗${RST}"
exit 1
