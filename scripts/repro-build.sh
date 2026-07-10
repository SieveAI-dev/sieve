#!/usr/bin/env bash
#
# repro-build.sh — Sieve 本地可复现构建验证（macOS only）
#
# 自证清白（可复现构建硬约束，.cursorrules §二 #6）：用户不应仅凭信任安装 Sieve，
# 而应能自己复现官方 release 二进制。本脚本镜像 .github/workflows/release.yml
# 的 reproducible-build job：
#   1. SOURCE_DATE_EPOCH = commit timestamp（消除构建时间污染）
#   2. RUSTFLAGS --remap-path-prefix（去除开发者本地路径污染）
#   3. cargo clean 后第一次 release 构建（锁定依赖；开发机 target/ 可能残留
#      增量产物，而 CI 是 fresh checkout，故本地 build1 也先 clean 求等价）
#   4. 再次 cargo clean 后第二次构建，比对两次 SHA-256 必须一致
#
# 用法：
#   ./scripts/repro-build.sh macos-arm64    # aarch64-apple-darwin
#   ./scripts/repro-build.sh macos-amd64    # x86_64-apple-darwin
#
# 退出码：
#   0  两次构建 SHA-256 一致（reproducible PASS）
#   1  哈希不一致 / 参数错误 / 环境不满足（FAIL）
#
# 比对官方 release：得到的 SHA-256 应与 GitHub Release 的 SHA256SUMS
# 中对应条目一致。任何差异 → 不要安装该二进制，立即在 GitHub Issues 报告。
# 实现细节见 docs/guides/deployment.md §3.3。

set -euo pipefail

# ─────────────────────────────────────────────────────────────
# 参数解析：平台别名 → rust target triple
# ─────────────────────────────────────────────────────────────
usage() {
  cat >&2 <<'EOF'
用法: repro-build.sh <platform>

platform:
  macos-arm64    aarch64-apple-darwin（Apple Silicon）
  macos-amd64    x86_64-apple-darwin（Intel Mac）

示例:
  ./scripts/repro-build.sh macos-arm64
EOF
  exit 1
}

if [ "$#" -ne 1 ]; then
  echo "错误: 需要且仅需要一个平台参数" >&2
  usage
fi

PLATFORM="$1"
case "$PLATFORM" in
  macos-arm64)
    TARGET="aarch64-apple-darwin"
    ;;
  macos-amd64)
    TARGET="x86_64-apple-darwin"
    ;;
  -h | --help)
    usage
    ;;
  *)
    echo "错误: 未知平台 '$PLATFORM'" >&2
    usage
    ;;
esac

# ─────────────────────────────────────────────────────────────
# 环境校验
# ─────────────────────────────────────────────────────────────
# Phase 1 仅支持 macOS（Tier 1 平台，见 deployment.md §3.3）。
if [ "$(uname -s)" != "Darwin" ]; then
  echo "错误: 本脚本仅支持 macOS（当前: $(uname -s)）。Linux / Windows 推 Phase 2。" >&2
  exit 1
fi

for cmd in git cargo rustup shasum; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "错误: 缺少依赖命令 '$cmd'，请先安装后重试" >&2
    exit 1
  fi
done

# 定位仓库根目录（脚本可从任意 cwd 调用）。
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." >/dev/null 2>&1 && pwd)"
cd "$REPO_ROOT"

if [ ! -f "Cargo.toml" ]; then
  echo "错误: 未在仓库根目录找到 Cargo.toml（REPO_ROOT=$REPO_ROOT）" >&2
  exit 1
fi

# 确保目标平台 toolchain 已安装（幂等，已装则 no-op）。
rustup target add "$TARGET" >/dev/null 2>&1 || true

# ─────────────────────────────────────────────────────────────
# 可复现构建环境（镜像 release.yml）
# ─────────────────────────────────────────────────────────────
# SOURCE_DATE_EPOCH = HEAD commit timestamp，消除构建时间戳污染。
SOURCE_DATE_EPOCH="$(git log -1 --format=%ct)"
export SOURCE_DATE_EPOCH

# --remap-path-prefix 把开发者本地绝对路径重写为固定前缀，
# 与 release.yml 保持一致（$HOME=/build, 仓库根=/src）。
export RUSTFLAGS="--remap-path-prefix=${HOME}=/build --remap-path-prefix=${REPO_ROOT}=/src"

BIN_PATH="target/${TARGET}/release/sieve"
OUT_DIR="target/repro"
OUT_BIN="${OUT_DIR}/sieve-${PLATFORM}"

echo "==> 平台:            $PLATFORM ($TARGET)"
echo "==> 仓库根:          $REPO_ROOT"
echo "==> SOURCE_DATE_EPOCH: $SOURCE_DATE_EPOCH ($(git log -1 --format=%h) $(git log -1 --format=%ci))"
echo

# ─────────────────────────────────────────────────────────────
# 第一次构建
# ─────────────────────────────────────────────────────────────
echo "==> [1/2] cargo clean 后第一次构建（cargo build --release --locked -j1）..."
# build1 必须先 cargo clean：开发机 target/ 常有增量产物，增量构建与 clean
# 构建可能字节不一致，会误报 reproducible FAIL（实为增量 vs clean 差异，
# 非真不可复现）。CI 是 fresh checkout 故天然 clean，本地补 clean 求等价。
# -j1 串行编译：fat-LTO 的合并顺序由并行编译完成顺序播种，并行下同源两次干净
# 构建产出双值 SHA-256（实测根因：lto=off 与 -j1 均收敛、thin/ld_classic 仍双值；
# codegen-units=1 已排除前端）。串行使合并顺序确定。【硬要求】release.yml 两步
# 构建必须同样带 -j1（或 CARGO_BUILD_JOBS=1），否则 CI 可复现 gate 会抖动中止 release。
cargo clean
cargo build --release --locked -j1 --target "$TARGET" -p sieve-cli
HASH1="$(shasum -a 256 "$BIN_PATH" | awk '{print $1}')"
echo "    build1 SHA-256: $HASH1"
echo

# ─────────────────────────────────────────────────────────────
# 清理后第二次构建
# ─────────────────────────────────────────────────────────────
echo "==> [2/2] cargo clean 后第二次构建（-j1 串行，理由见 build1）..."
cargo clean
cargo build --release --locked -j1 --target "$TARGET" -p sieve-cli
HASH2="$(shasum -a 256 "$BIN_PATH" | awk '{print $1}')"
echo "    build2 SHA-256: $HASH2"
echo

# ─────────────────────────────────────────────────────────────
# SHA-256 比对（hard gate）
# ─────────────────────────────────────────────────────────────
echo "==> 比对两次构建 SHA-256 ..."
echo "    build1: $HASH1"
echo "    build2: $HASH2"
if [ "$HASH1" != "$HASH2" ]; then
  echo >&2
  echo "FAIL: 两次构建哈希不一致，构建不可复现。" >&2
  echo "      请检查本地工具链版本是否与官方 release 一致，" >&2
  echo "      或在 GitHub Issues 报告（附 rustc -V / cargo -V / 本机平台）。" >&2
  exit 1
fi

# 一致时保留单一最终产物。
# 上一步 cargo clean 已删除 target/（含 OUT_DIR），cp 前必须重建目录，
# 否则 cp 落空报错（历史 bug：mkdir 只在 build1 前做一次，被 clean 抹掉）。
mkdir -p "$OUT_DIR"
cp "$BIN_PATH" "$OUT_BIN"
chmod +x "$OUT_BIN"

echo
echo "PASS: 可复现构建通过。"
echo "      SHA-256: $HASH1"
echo "      产物:    $OUT_BIN"
echo
echo "下一步：与官方 GitHub Release 的 SHA256SUMS 对应条目比对，"
echo "        一致才安装该二进制（deployment.md §3.3）。"
