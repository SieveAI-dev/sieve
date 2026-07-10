#!/usr/bin/env bash
#
# check_routing_coverage.sh — content-type 四路由覆盖门禁（.cursorrules §二 #16）
#
# 永久化 v1.5.4 P0 教训：入站检测必须覆盖全部 4 条 content-type 路由——
#   M-1 Anthropic SSE / M-2 Anthropic JSON / M-3 OpenAI SSE / M-4 OpenAI JSON。
# 「只挂 SSE 不挂 JSON」= P0 漏洞。
#
# 守护方式（比"grep 测试体找 rule_id"的朴素方案更稳健：直接在源码侧拦截
# bug 形状，不依赖测试命名约定，且对历史欠债不误报）：
#   检查 A（源码侧，核心）—— 四个路由 handler 都接了入站检测钩子：
#     · 两条 JSON handler 必须同时调 on_tool_use_complete + scan_assistant_text
#     · 共享 SSE 分类器 classify_inbound_detections 必须调 observe_event + on_tool_use_complete
#     · 两条 SSE forward 函数必须接入 classify_inbound_detections
#   检查 B（测试侧）—— 四路由端到端测试锚点齐全且未被 #[ignore]。
#
# 退出码：0=通过；1=覆盖缺口（CI 阻断合并）；2=脚本/环境错误。
#
# 范围说明：per-rule × 4-route 的"全量测试矩阵"（更长期的"测试量翻倍"目标，
# 存在历史 M-2/M-3 欠债）是更高一层、单独排期的工作，不在本门禁内。本门禁只守护
# 不可回退的【结构不变量】：任一路由的入站检测钩子被摘除 → 立即 CI 失败。
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

DAEMON="crates/sieve-cli/src/daemon.rs"
MATRIX="crates/sieve-cli/tests/content_type_matrix.rs"
verbose="${1:-}"
fail=0

[ -f "$DAEMON" ] || { echo "✗ 找不到 $DAEMON" >&2; exit 2; }
[ -f "$MATRIX" ] || { echo "✗ 找不到 $MATRIX" >&2; exit 2; }

# 把每个函数体内出现的钩子调用，归属到「最近一次 fn 定义」（闭包 async move {…} 不重置归属，
# 故 spawn 内的调用仍算在外层函数名下）。输出去重的 "函数名|钩子名" 集合。
PAIRS="$(awk '
  /^[[:space:]]*(pub[[:space:]]+)?(async[[:space:]]+)?fn[[:space:]]+[A-Za-z0-9_]+/ {
    line=$0
    sub(/^.*fn[[:space:]]+/, "", line)
    sub(/[^A-Za-z0-9_].*$/, "", line)
    cur=line
  }
  cur=="" { next }
  /observe_event/               { print cur "|observe_event" }
  /scan_assistant_text/         { print cur "|scan_assistant_text" }
  /on_tool_use_complete/        { print cur "|on_tool_use_complete" }
  /classify_inbound_detections/ { print cur "|classify_inbound_detections" }
  /handle_json_inbound/         { print cur "|handle_json_inbound" }
' "$DAEMON" | sort -u)"

has_pair() { grep -qxF "$1|$2" <<<"$PAIRS"; }

require() {  # require <fn> <call> <人类描述>
  if has_pair "$1" "$2"; then
    if [ "$verbose" = "-v" ]; then echo "  ✓ $1 → $2"; fi
  else
    echo "✗ FAIL[源码]: 函数 $1 未调用 $2 —— $3"
    echo "             （四路由不变量破坏，v1.5.4 P0 形状；见 .cursorrules §二 #16）"
    fail=1
  fi
}

echo "── 检查 A：四路由入站钩子接线（${DAEMON}）──"
# A1/A2 网关三层重构后：JSON 两路由（M-2/M-4）收敛到 codec 驱动的单一 handle_json_inbound，
# SSE 两路由（M-1/M-3）经各自 forward 进共享 classify_inbound_detections。不变量不变：
# 四条路由都必须接入站文本扫描 + 工具检测钩子。
require handle_json_inbound on_tool_use_complete "M-2/M-4 JSON 工具检测（codec 驱动统一 handler）"
require handle_json_inbound scan_assistant_text  "M-2/M-4 JSON 文本扫描 / IN-CR-01"
require classify_inbound_detections observe_event        "M-1/M-3 SSE 文本扫描 / IN-CR-01"
require classify_inbound_detections on_tool_use_complete "M-1/M-3 SSE 工具检测"
require forward_with_inbound_inspection        classify_inbound_detections "M-1 Anthropic SSE 接入分类器"
require forward_with_inbound_inspection        handle_json_inbound         "M-2 Anthropic JSON 进统一 handler"
require forward_with_openai_inbound_inspection classify_inbound_detections "M-3 OpenAI SSE 接入分类器"
require forward_with_openai_inbound_inspection handle_json_inbound         "M-4 OpenAI JSON 进统一 handler"

echo "── 检查 B：四路由端到端测试锚点（${MATRIX}）──"
for route in anthropic_sse anthropic_json openai_sse openai_json; do
  fn_line="$(grep -nE "fn content_type_matrix_${route}([^A-Za-z0-9_]|\$)" "$MATRIX" | head -1 | cut -d: -f1 || true)"
  if [ -z "$fn_line" ]; then
    echo "✗ FAIL[测试]: 缺四路由端到端测试 content_type_matrix_${route}（M 路由失覆盖）"
    fail=1
  elif sed -n "$((fn_line - 1))p" "$MATRIX" | grep -q '#\[ignore'; then
    echo "✗ FAIL[测试]: content_type_matrix_${route} 被 #[ignore]（M 路由测试形同失覆盖）"
    fail=1
  else
    if [ "$verbose" = "-v" ]; then echo "  ✓ content_type_matrix_${route}"; fi
  fi
done

echo ""
if [ "$fail" -eq 0 ]; then
  echo "✓ 四路由覆盖门禁通过（M-1~M-4 入站钩子接线 + 端到端测试齐全）"
else
  echo "✗ 四路由覆盖门禁失败：上述路由缺入站检测钩子或测试。参见 .cursorrules §二 #16。"
fi
exit "$fail"
