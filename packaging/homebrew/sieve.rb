# Homebrew formula — Sieve CLI/daemon 二进制。
#
# 复制到独立 tap 仓库 SieveAI-dev/homebrew-sieve 的 Formula/sieve.rb（见本目录 README.md）。
# 用户安装：
#   brew tap SieveAI-dev/sieve
#   brew install sieve
#
# brew 用 formula 内的 sha256 自动校验下载产物——校验是包管理器原生提供的，无需用户手动验签。
#
# ⚠️ pre-GA 占位：首个 release(vX.Y.Z) 发布后，用 release 的 SHA256SUMS 填入真实 sha256，
#    并把 version 改为该 release。占位的全零 sha256 会让 brew install 校验失败（fail-closed）。
class Sieve < Formula
  desc "Local-only LLM traffic security proxy for crypto developers (CLI/daemon)"
  homepage "https://github.com/SieveAI-dev/sieve"
  version "0.1.0-alpha"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/SieveAI-dev/sieve/releases/download/v#{version}/sieve-aarch64-apple-darwin"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000" # REPLACE: aarch64
    end
    on_intel do
      url "https://github.com/SieveAI-dev/sieve/releases/download/v#{version}/sieve-x86_64-apple-darwin"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000" # REPLACE: x86_64
    end
  end

  def install
    # release 产物是裸二进制（名带 target triple），装为 bin/sieve。
    bin.install Dir["sieve-*-apple-darwin"].first => "sieve"
  end

  test do
    assert_match "sieve", shell_output("#{bin}/sieve --version")
  end
end
