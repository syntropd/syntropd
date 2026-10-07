#!/usr/bin/env bash
# Focused test for install.sh model provisioning. Runs fully unprivileged
# via --dry-run (never touches the system). Fails on the first mismatch.
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALL="${REPO_DIR}/install.sh"
SITE_COPY="/home/ubermetroid/Projects/syntropd.github.io/install.sh"

pass() { echo "PASS: $1"; }

bash -n "${INSTALL}"
pass "syntax"

"${INSTALL}" --help | grep -q -- "--no-models" || { echo "FAIL: --help hides model flags"; exit 1; }
pass "--help documents model flags"

"${INSTALL}" --help | grep -q -- "--quiet" || { echo "FAIL: --help hides --quiet"; exit 1; }
"${INSTALL}" --help | grep -q -- "--verbose" || { echo "FAIL: --help hides --verbose"; exit 1; }
pass "--help documents verbosity flags"

"${INSTALL}" --help | grep -q -- "--hf-token" || { echo "FAIL: --help hides --hf-token"; exit 1; }
pass "--help documents --hf-token"

out="$(HF_TOKEN="" "${INSTALL}" --dry-run --hf-token custom_test_token 2>&1)"
echo "${out}" | grep -q "Hugging Face credentials" || { echo "FAIL: dry-run ignores --hf-token"; exit 1; }
pass "dry-run accepts --hf-token"

out="$("${INSTALL}" --dry-run 2>&1)"
echo "${out}" | grep -q "gemma-4-E2B-it-Q4_K_M.gguf\|Qwen starter brain instead" \
  || { echo "FAIL: default dry-run picks no brain"; exit 1; }
pass "default dry-run picks a fitting brain"
echo "${out}" | grep -q "Brain fit" || { echo "FAIL: dry-run hides fit decision"; exit 1; }
pass "dry-run reports the RAM fit decision"

out="$(TEST_MEM_KB=8000000 "${INSTALL}" --dry-run 2>&1)"
echo "${out}" | grep -q "Qwen starter brain instead" || { echo "FAIL: small RAM not downgraded"; exit 1; }
echo "${out}" | grep -q "qwen2.5-0.5b-instruct-q8_0.gguf" || { echo "FAIL: downgrade misses Qwen"; exit 1; }
pass "small RAM auto-downgrades to Qwen"

out="$(TEST_MEM_KB=8000000 "${INSTALL}" --dry-run --with-gemma 2>&1)"
echo "${out}" | grep -q "gemma-4-E2B-it-Q4_K_M.gguf" || { echo "FAIL: explicit Gemma not honored"; exit 1; }
echo "${out}" | grep -q "expect load failure" || { echo "FAIL: explicit Gemma unwarned"; exit 1; }
pass "explicit --with-gemma wins with a warning"

# Either line proves the wiring step ran (outcome depends on local files).
echo "${out}" | grep -q "routerctl setup --auto\|skipping router wiring" \
  || { echo "FAIL: dry-run misses router wiring"; exit 1; }
pass "dry-run covers router wiring"

out="$("${INSTALL}" --dry-run --no-models 2>&1)"
echo "${out}" | grep -q "engine only" || { echo "FAIL: --no-models not honored"; exit 1; }
echo "${out}" | grep -q "gemma-4-E2B" && { echo "FAIL: --no-models still lists Gemma"; exit 1; }
pass "--no-models skips downloads"

out="$("${INSTALL}" --dry-run --with-starter-model --with-vision 2>&1)"
for f in "qwen2.5-0.5b-instruct-q8_0.gguf" "mmproj-F16.gguf" "gemma-4-E2B-it-Q4_K_M.gguf"; do
  echo "${out}" | grep -q "${f}" || { echo "FAIL: dry-run misses ${f}"; exit 1; }
done
pass "opt-in flags add starter + vision (Gemma stays default)"

if "${INSTALL}" --bogus-flag >/dev/null 2>&1; then echo "FAIL: unknown flag accepted"; exit 1; fi
pass "unknown flag rejected"

out="$("${INSTALL}" --dry-run --quiet 2>&1)"
echo "${out}" | grep -q "Native AI Subsystem" || { echo "FAIL: quiet hides banner"; exit 1; }
echo "${out}" | grep -q "\[INFO\]" && { echo "FAIL: quiet leaks trivia"; exit 1; }
echo "${out}" | grep -q "\[OK\]" && { echo "FAIL: quiet leaks trivia"; exit 1; }
pass "--quiet suppresses trivia, keeps banner"

if grep -qi "ollama" "${INSTALL}"; then echo "FAIL: installer mentions Ollama"; exit 1; fi
pass "installer never mentions Ollama"

# Unprivileged daemons: each generated unit must carry its User= line.
for spec in \
  "toold.service:User=syntrop-tool" \
  "contextd.service:User=syntrop-context" \
  "inferenced.service:User=inferenced" \
  "modeld.service:User=modeld" \
; do
  unit="${spec%%:*}"; want="${spec##*:}"
  awk "/UNIT_DIR}\/${unit}\"/,/^EOF\$/" "${INSTALL}" | grep -q "^${want}\$" \
    || { echo "FAIL: generated ${unit} lacks ${want}"; exit 1; }
done
pass "generated units run unprivileged"

grep -q 'polkit-1/rules.d/49-syntrop-tool.rules' "${INSTALL}" \
  || { echo "FAIL: installer ships no toold polkit rule"; exit 1; }
pass "installer ships toold polkit rule"

grep -q 'polkit-1/rules.d/50-syntrop-inhibit.rules' "${INSTALL}" \
  || { echo "FAIL: installer ships no inhibitor polkit rule"; exit 1; }
pass "installer ships inhibitor polkit rule"

grep -q '/usr/lib/syntrop/bin/systemd-inhibit' "${INSTALL}" \
  || { echo "FAIL: installer ships no systemd-inhibit shim"; exit 1; }
pass "installer ships systemd-inhibit shim"

awk "/UNIT_DIR}\/inferenced.service\"/,/^EOF\$/" "${INSTALL}" | grep -q 'Environment="PATH=/usr/lib/syntrop/bin' \
  || { echo "FAIL: generated inferenced.service lacks inhibitor shim PATH"; exit 1; }
pass "generated inferenced.service contains inhibitor shim PATH"

grep -q 'syntrop-uninstall' "${INSTALL}" \
  || { echo "FAIL: installer does not install syntrop-uninstall"; exit 1; }
pass "installer provisions syntrop-uninstall"

UNINSTALL="${REPO_DIR}/uninstall.sh"
SITE_UNINSTALL="/home/ubermetroid/Projects/syntropd.github.io/uninstall.sh"

bash -n "${UNINSTALL}"
pass "uninstall.sh syntax"

"${UNINSTALL}" --help | grep -q -- "--purge" || { echo "FAIL: uninstaller hides --purge"; exit 1; }
"${UNINSTALL}" --help | grep -q -- "--dry-run" || { echo "FAIL: uninstaller hides --dry-run"; exit 1; }
pass "uninstaller documents flags"

out_un="$("${UNINSTALL}" --dry-run 2>&1)"
echo "${out_un}" | grep -q "Uninstallation dry-run complete" || { echo "FAIL: dry-run uninstall failed"; exit 1; }
echo "${out_un}" | grep -q "Preserved Assets" || { echo "FAIL: dry-run default does not preserve models"; exit 1; }
pass "uninstaller default preserves assets"

out_purge="$("${UNINSTALL}" --dry-run --purge 2>&1)"
echo "${out_purge}" | grep -q "Purging Data and Accounts" || { echo "FAIL: dry-run purge failed"; exit 1; }
pass "uninstaller purge targets data and accounts"

if [[ -f "${SITE_UNINSTALL}" ]]; then
  cmp -s "${UNINSTALL}" "${SITE_UNINSTALL}" || { echo "FAIL: site uninstall.sh out of sync"; exit 1; }
  pass "site uninstall.sh identical"
fi

if [[ -f "${SITE_COPY}" ]]; then
  cmp -s "${INSTALL}" "${SITE_COPY}" || { echo "FAIL: site install.sh out of sync"; exit 1; }
  pass "site copy identical"
else
  echo "SKIP: site checkout not present; cannot check sync"
fi

echo "ALL INSTALLER CHECKS PASSED"
