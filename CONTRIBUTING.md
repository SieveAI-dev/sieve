中文贡献者请先读 [README.zh-CN.md](./README.zh-CN.md)

# Contributing to Sieve

Thanks for your interest in contributing. Sieve is a **fully local, crypto-native LLM-traffic security proxy** — a single Rust binary that sits between your AI coding agent (Claude Code / Codex CLI / Cursor) and the upstream model API (Anthropic / OpenAI / relays). It inspects traffic in both directions: redacting secrets on the way out, and blocking dangerous tool calls on the way in (fail-closed), to force a moment of cognitive friction before irreversible actions (signing, transfers, deploys).

Because Sieve is a **security product**, contributions are held to a higher bar than a typical project: the detection engine runs **100% locally**, the trust boundary is narrow and explicit, and a set of hard constraints (below) is non-negotiable. Please read this whole document before opening a PR.

> **Project status:** the repository is **public** and in **pre-GA closed beta** (invite-only testers). Source is public so the trust story is *verifiable, not merely asserted*.

---

## Scope

This repository (`SieveAI-dev/sieve`) is the **Rust daemon** only. The macOS GUI lives in a separate repo, [`SieveAI-dev/sieve-gui-macos`](https://github.com/SieveAI-dev/sieve-gui-macos) (SwiftUI) — file GUI issues and PRs there.

Good contributions here:

- Detection rules and engine improvements (with tests + fuzz coverage where applicable)
- SSE / protocol robustness, tool-call aggregation, inbound filtering
- Real-world attack reproduction samples (see *Submitting samples*)
- Bug fixes, performance, docs, and CI hardening

---

## Prerequisites

- **Rust toolchain** — pinned via [`rust-toolchain.toml`](./rust-toolchain.toml) (currently `1.88.0` with `rustfmt` + `clippy`). `rustup` will install the right toolchain automatically when you build inside the repo; do **not** override the channel.
- **`cargo-deny`** — for license / advisory / source auditing:

  ```bash
  cargo install cargo-deny --locked
  ```

- **Nightly toolchain + `cargo-fuzz`** — required only if your change touches SSE parsing, rules, or tool-call decisioning (see *Fuzzing*):

  ```bash
  rustup toolchain install nightly
  cargo install cargo-fuzz --locked
  ```

---

## Build & test

Run the full local gate before opening a PR. CI runs the same commands and treats warnings as errors.

```bash
# Build the whole workspace, locked to Cargo.lock
cargo build --workspace --locked

# Formatting (must pass, no diffs)
cargo fmt --all -- --check

# Lint — warnings are errors
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings

# Supply-chain / license / advisory audit
cargo deny check

# Tests
cargo test --workspace --locked
```

Sieve ships with an **extensive test suite** (unit, integration, and end-to-end), including **real attack reproduction samples**. Keep it green: a PR that touches detection logic without corresponding tests will not be merged.

---

## Fuzzing

**Any PR that touches SSE parsing, detection rules, or tool-call decisioning MUST include fuzz coverage.** This is a hard constraint (PRD §9 #5) — *a PR without fuzz coverage for these paths will not be merged.*

Run the relevant targets with the nightly toolchain:

```bash
cargo +nightly fuzz run sse_parser
cargo +nightly fuzz run tool_use_aggregator
cargo +nightly fuzz run inbound_filter
```

(An additional `sse_parser_openai` target also exists for OpenAI-protocol SSE.) When you add a new parsing or decision boundary, add or extend a fuzz target rather than relying on example-based tests alone. SSE edge cases that must stay covered: half-line chunks, separators split across chunks, embedded C0 control characters, multiple events coalesced in one chunk, and early stream termination.

---

## Hard constraints (do not relax)

Sieve enforces **16 non-negotiable engineering constraints** (PRD §9), mirrored in [`CLAUDE.md`](./CLAUDE.md) and [`.cursorrules`](./.cursorrules). They are *not* optimization knobs — each one is "get it wrong and the product is dead." A few that contributors hit most often:

- **Detection inference is 100% local** — there is **no networked verifier**. The only permitted outbound traffic is: (1) the upstream LLM API the user explicitly called, (2) the update manifest check (`updates.sieveai.dev`, ~4×/day, 5 fields, can be disabled), and (3) rule-body fetches (`cdn.sieveai.dev`). Never send a token, key, prompt, or response anywhere to be "checked."
- **Fail-closed High-Risk Tool Policy Gate** — Critical tool calls (signing / shell / sensitive paths) require human confirmation and **cannot be disabled in any mode**, including YOLO mode.
- **Critical false-positive rate must stay below the product axiom threshold**, and **Critical detection can never be turned off** in any build, including degraded mode.
- **No protocol-layer lying** — never forge `tool_use` / `stop_reason` / `id` / `usage` / `type`. Injecting a self-reported `sieve_blocked` SSE event is allowed; impersonating the model is not.
- **No local CA / MITM**, **outbound redaction must not interrupt the workflow** (auto-redact + status-bar notice, no modal storms), and **user rules are fail-safe** (they can never override or suppress a system Critical).

The full list lives in **[`CLAUDE.md` → "不可放宽的硬约束"](./CLAUDE.md)** and **[`.cursorrules` §二](./.cursorrules)**. **If your change touches any of these, stop and open a discussion first** — relaxing a hard constraint requires explicit maintainer sign-off and an ADR.

---

## Commit & PR conventions

- **Conventional Commits**: `feat:`, `fix:`, `docs:`, `refactor:`, `chore:`, `test:`, etc.
- **One commit does one thing** — do not mix functional changes with reformatting.
- Keep PRs focused and small enough to review.
- Update related docs and `CHANGELOG` when behavior, dependencies, or detection IDs change (see the doc-update tables in `CLAUDE.md` / `.cursorrules`).

### No AI signatures — mandatory

**Commits and PRs must not contain any AI attribution of any kind.** This is a hard rule with no exceptions:

- No `Co-Authored-By:` lines referencing any AI tool
- No `Generated with …` / "🤖 Generated with …" footers or bylines in commit messages or PR descriptions
- No AI-generated signature, trailer, or emoji byline anywhere in the commit history

Any such line will be rejected in review. Keep the git history clean and human-attributed.

---

## Reporting security issues

**Do not open a public issue for vulnerabilities.** Follow the process in [`SECURITY.md`](./SECURITY.md) for private disclosure. Sieve is a security product; responsible disclosure protects all beta users.

---

## Submitting samples

If you have a suspicious prompt, response, or tool-call that Sieve should catch (or wrongly flags), submit it via the **`suspicious_sample`** issue template under [`.github/ISSUE_TEMPLATE/`](./.github/ISSUE_TEMPLATE/suspicious_sample.md). Redact your own secrets before submitting — never paste a real private key, seed phrase, or API key into a public issue.

---

## License of contributions

By contributing, you agree that your contributions are licensed under the project's **inbound = outbound** terms:

- **Code** contributions are accepted under the **Apache License 2.0** (the project's code license; see [`LICENSE`](./LICENSE) and ADR-035).
- **Documentation** contributions (everything under `docs/`, plus `README*` / `CLAUDE.md` and other non-source Markdown) are accepted under **CC BY-NC-SA 4.0**.

If you are contributing on behalf of an employer, make sure you have the right to do so under these terms.
