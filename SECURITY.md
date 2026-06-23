# Security Policy

> Sieve is a security proxy that sits in your LLM traffic (PRD §1.1). If Sieve itself is
> compromised, its entire value proposition collapses. "Verifiable, not merely trusted" is
> the core of its security promise (PRD §1.2).
>
> **Please do not report vulnerabilities in Sieve through public GitHub Issues.** Use the
> private channel below.

---

## Reporting a vulnerability

Report privately through **GitHub Security Advisories** — no email involved:

1. Open the repository's **[Security tab → Report a vulnerability](https://github.com/SieveAI-dev/sieve/security/advisories/new)**.
2. GitHub creates a **private** advisory thread (not visible to the public) that the maintainers
   triage directly. No email address is exposed, and nothing is disclosed until a fix ships.

> Why a private advisory rather than a public issue or email? A public issue would disclose the
> flaw before a fix exists — exactly what a security tool must avoid. A private GitHub advisory
> keeps the report confidential, tracked, and tied to a coordinated fix. (Non-sensitive security
> questions — e.g. a suspected false positive — can go in a normal [issue](https://github.com/SieveAI-dev/sieve/issues).)

Please include:

- **Affected version**: output of `sieve --version` (binary SHA-256 + rules version)
- **Platform**: OS / arch / install method (brew / binary / source build)
- **Vulnerability class** (any of):
  - Supply chain (tampered binary / rules package / dependency)
  - Data exfiltration (prompt upload / remote verifier)
  - Fail-closed bypass (YOLO-mode bypass / Critical interception disabled)
  - Detection bypass (a known attack pattern that does not trigger)
  - Missing config-layer validation (e.g. `bind_address = 0.0.0.0` not rejected)
  - Denial of service (crash / memory blow-up / deadlock)
- **Reproduction** (minimal case)
- **Impact assessment**: the asset-loss risk it could lead to
- **Credit preference**: GitHub username, or anonymous

---

## Response-time commitments

| Stage | SLA |
|-------|-----|
| Acknowledge the advisory | **within 24 hours** |
| Initial assessment (severity + reproduction) | **within 7 days** |
| Fix or mitigation (by severity) | Critical: 7 days / High: 30 days / Medium: 90 days |
| Public advisory | within 30 days of the fix shipping |

> The maintainer team is small and the SLAs above reflect that. If the issue is **actively
> exploited with asset-loss risk**, prefix the advisory title with `[URGENT]` and it will be
> prioritized.

---

## Coordinated disclosure

- Please do not disclose before a fix ships (talks, blog posts, social media, and vulnerability
  databases included).
- The reporter is credited when the fix ships (unless you ask to stay anonymous).
- If user asset-loss risk is involved, a forced upgrade is pushed and the event is published via
  the transparency log.
- No cash bounty for now, but significant findings are credited in the post-GA advisory and
  write-up.

---

## Out of scope

The following are not Sieve security vulnerabilities:

- **User misconfiguration**: e.g. setting `[server].bind_address` to `0.0.0.0` (the config layer
  refuses to start).
- **Relay / upstream API vulnerabilities**: outside Sieve's remit (these are precisely what Sieve
  is built to detect, PRD §1.2).
- **Wallet / browser-extension phishing**: Sieve is a cognitive-friction layer, not a wallet
  security product.
- **Detection false positives / negatives** (unless they violate the PRD §6.5 FP budget): FP
  handling goes through [`.sieveignore`](docs/api/api-reference.md) and the normal GitHub issue flow.
- **Using an expired / unsigned binary**: verifying the [sigstore signature](docs/guides/deployment.md)
  is the user's responsibility at install time — and the installer does it for you automatically.

---

## Our supply-chain commitments

Sigstore signing + Reproducible Build + transparency log:

- Every release binary is **sigstore-signed + logged in Rekor** (verifiable with `cosign verify-blob`).
- **Tier 1 (macOS / Linux) reproducible builds**: two independent builds must produce an identical
  SHA-256 before release.
- **Tier 2 (Windows)**: binary + sigstore signature; reproducible build deferred to Phase 2.
- Rules packages are **Ed25519-signed + fail-closed verified** (on signature failure, the last
  verified ruleset is kept). Alpha builds ship a placeholder key and are temporarily fail-open
  (skip+warn, backed by same-origin SHA-256); **GA builds enforce a real key at compile time via
  the `ga_keys` gate** — a placeholder fails to compile.
- **Pinned dependencies**: `Cargo.lock` committed + Dependabot weekly (major bumps reviewed individually).

Suggested supply-chain audit:

```bash
# 1. Verify the binary signature
cosign verify-blob \
  --certificate-identity-regexp '^https://github.com/SieveAI-dev/sieve/' \
  --certificate-oidc-issuer 'https://token.actions.githubusercontent.com' \
  --bundle <artifact>.sigstore.json \
  ./sieve

# 2. Reproduce the build and compare SHA-256 (Tier 1)
git clone https://github.com/SieveAI-dev/sieve.git --branch v0.1.0
./scripts/repro-build.sh linux-amd64
sha256sum target/repro/sieve-linux-amd64
sha256sum ./sieve   # must match

# 3. Confirm "no networked verifier" by capturing traffic
sudo tcpdump -i any -nn host '!api.anthropic.com and !your-relay.com'
sieve --config ~/.sieve/config.toml
# expected: no outbound traffic except the upstream API
```

---

## Past advisories

> Pre-GA advisories are not assigned formal `SIEVE-YYYY-NNN` IDs; they are recorded in the
> CHANGELOG. Formal IDs start post-GA.

### Pre-GA P0: non-streaming JSON inbound detection bypass (fixed 2026-05-01)

- **Affected versions**: v1.5.0 ~ v1.5.3 (v1.5.x, 70 inbound rules)
- **Scope**:
  - **Bug 1 (Anthropic)**: `tool_use` inside an `application/json` non-streaming response bypassed
    all inbound rules (IN-CR-02/03/04/05 / IN-GEN-* all inert).
  - **Bug 2 (OpenAI)**: the `proxy_openai` `stream=false` branch skipped inbound detection; OpenAI
    defaults to `stream=false`, so OpenAI inbound rules had **never** taken effect.
- **Severity**: P0 (a severe product-level defect, given that inbound detection is Sieve's core
  capability — PRD §5.2).
- **Fix**: v1.5.4, commit `14153e2`, see the [CHANGELOG](docs/changelog/CHANGELOG.md).
- **Fix verification**: 2 new integration tests + dataset FP rate 0% / 99.71% recall, no regression.
- **Disclosure status**: found and fixed during pre-GA internal dogfood; both discovery and fix
  predate any public release, so no external users were affected.

---

## Related documents

- PRD §9 engineering hard constraints
- [Deployment guide — binary signature verification](docs/guides/deployment.md)
