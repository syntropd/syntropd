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

if [[ -f "${SITE_COPY}" ]]; then
  cmp -s "${INSTALL}" "${SITE_COPY}" || { echo "FAIL: site install.sh out of sync"; exit 1; }
  pass "site copy identical"
else
  echo "SKIP: site checkout not present; cannot check sync"
fi

echo "ALL INSTALLER CHECKS PASSED"
