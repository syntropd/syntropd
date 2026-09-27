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
echo "${out}" | grep -q "gemma-4-E2B-it-Q4_K_M.gguf" || { echo "FAIL: default dry-run misses Gemma"; exit 1; }
pass "default dry-run ships the Gemma brain"

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
