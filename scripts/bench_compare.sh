#!/usr/bin/env bash
# bench_compare.sh — 对比 criterion PR baseline 与 main baseline 的 mean 时间
#
# 用法：
#   THRESHOLD_PCT=10 ./scripts/bench_compare.sh
#
# 降级说明：
#   criterion 0.5 的 estimates.json 不包含独立的 P99 字段，P99 需要从
#   sample.json 的原始采样数据自行计算（sample 量少时 P99 噪声极大）。
#   CI 以 mean.point_estimate（纳秒）做代理指标，threshold 10% 不变。
#   此选择有意保守——10% mean 退化对实时系统已是明显信号，
#   且均值受尾部影响，比 median 更敏感于异常慢样本。
#
# 退出码：
#   0 — 所有 bench 均在 threshold 内
#   1 — 任一 bench 退化超过 threshold，或脚本自身错误
#
# 依赖：jq, awk（macOS/Linux 均预装）
set -euo pipefail

THRESHOLD_PCT=${THRESHOLD_PCT:-10}
CRITERION_DIR=${CRITERION_DIR:-target/criterion}
PR_BASELINE=${PR_BASELINE:-pr}
MAIN_BASELINE=${MAIN_BASELINE:-main}

if ! command -v jq &>/dev/null; then
    echo "错误：需要 jq，请先安装（brew install jq / apt install jq）" >&2
    exit 1
fi

found=0
failed=0

# 遍历所有 pr baseline 的 estimates.json
# 路径格式：target/criterion/<group>/<id>/pr/estimates.json
while IFS= read -r -d '' pr_file; do
    # 找对应的 main baseline
    bench_dir=$(dirname "$(dirname "$pr_file")")          # target/criterion/<group>/<id>
    main_file="${bench_dir}/${MAIN_BASELINE}/estimates.json"

    if [[ ! -f "$main_file" ]]; then
        bench_label="${bench_dir#"${CRITERION_DIR}"/}"
        echo "skip（无 main baseline）: ${bench_label}"
        continue
    fi

    found=$((found + 1))
    bench_label="${bench_dir#"${CRITERION_DIR}"/}"

    pr_mean=$(jq '.mean.point_estimate' "$pr_file")
    main_mean=$(jq '.mean.point_estimate' "$main_file")

    if [[ "$main_mean" == "0" ]] || [[ "$main_mean" == "null" ]]; then
        echo "skip（main mean = 0 或 null）: ${bench_label}"
        continue
    fi

    # delta_pct = (pr - main) / main * 100，正值表示退化
    delta_pct=$(awk "BEGIN{ printf \"%.2f\", ($pr_mean - $main_mean) / $main_mean * 100 }")

    # 换算为 µs，方便阅读
    pr_us=$(awk "BEGIN{ printf \"%.1f\", $pr_mean / 1000 }")
    main_us=$(awk "BEGIN{ printf \"%.1f\", $main_mean / 1000 }")

    status="OK"
    if awk "BEGIN{ exit !($delta_pct > $THRESHOLD_PCT) }"; then
        status="FAIL"
        failed=$((failed + 1))
    fi

    printf "[%s] %s — PR=%.1fµs main=%.1fµs delta=%+.2f%%\n" \
        "$status" "$bench_label" "$pr_us" "$main_us" "$delta_pct"
done < <(find "${CRITERION_DIR}" -path "*/${PR_BASELINE}/estimates.json" -print0 2>/dev/null)

if [[ $found -eq 0 ]]; then
    echo "警告：未找到任何 PR baseline（${PR_BASELINE}），请先跑 cargo bench -- --save-baseline ${PR_BASELINE}" >&2
    exit 1
fi

echo "---"
echo "共检查 ${found} 个 bench，threshold=${THRESHOLD_PCT}%"

if [[ $failed -gt 0 ]]; then
    echo "FAIL: ${failed} 个 bench 超过退化阈值" >&2
    exit 1
fi

echo "all bench within ${THRESHOLD_PCT}% threshold"
