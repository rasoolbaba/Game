#!/usr/bin/env bash
# Runs all four implementations at the same T, prints each result, and ASSERTS that
# every implementation produced the bit-identical fingerprint (determinism gate).
set -euo pipefail
T="${1:-200000}"
cd "$(dirname "$0")"

RN=$(./rust/sim "$T")
RW=$(node rust/run_wasm.mjs "$T")
AS=$(cd as && node run.mjs "$T")
JS=$(node js/sim.mjs "$T")

echo "$RN"; echo "$RW"; echo "$AS"; echo "$JS"

a() { jq -r .acc <<<"$1"; }
A1=$(a "$RN"); A2=$(a "$RW"); A3=$(a "$AS"); A4=$(a "$JS")
echo "---"
if [ "$A1" = "$A2" ] && [ "$A1" = "$A3" ] && [ "$A1" = "$A4" ]; then
  echo "DETERMINISM @ T=$T: PASS ✅  (all four acc == $A1)"
else
  echo "DETERMINISM @ T=$T: FAIL ❌  rust-native=$A1 rust-wasm=$A2 as-wasm=$A3 js-bigint=$A4"
  exit 1
fi