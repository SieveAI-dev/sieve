# Homebrew cask — Sieve GUI (.app)。
#
# 复制到独立 tap 仓库 SieveAI-dev/homebrew-sieve 的 Casks/sieve.rb（见本目录 README.md）。
# 用户安装：
#   brew install --cask sieve
#
# brew 用 sha256 自动校验下载的 .dmg。GUI 不走 curl|sh —— 走签名 .dmg / 此 cask。
#
# ⚠️ pre-GA 占位：GUI 仓(sieve-gui-macos)首个 release 后填真实 .dmg sha256 + 改 version。
cask "sieve" do
  version "0.1.0"
  sha256 "0000000000000000000000000000000000000000000000000000000000000000" # REPLACE: .dmg sha256

  url "https://github.com/SieveAI-dev/sieve-gui-macos/releases/download/v#{version}/SieveGUI-#{version}.dmg"
  name "Sieve GUI"
  desc "Local-only LLM traffic security proxy for crypto developers (GUI)"
  homepage "https://github.com/SieveAI-dev/sieve"

  depends_on macos: ">= :ventura"

  app "SieveGUI.app"

  zap trash: [
    "~/Library/Caches/sieve",
    "~/.sieve",
  ]
end
