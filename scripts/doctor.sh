#!/usr/bin/env bash
set -euo pipefail

missing=0
for tool in rustup cargo rustc just java ktlint google-java-format node pnpm; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "Missing prerequisite: $tool" >&2
        missing=1
    fi
done
if [ "$missing" -ne 0 ]; then
    echo "Run mise install and activate mise, or use mise exec -- just doctor. See README.md." >&2
    exit 1
fi

cargo --version
rustc --version
just --version
java -version
ktlint --version
google-java-format --version
node --version
pnpm --version
if ! pnpm exec oxfmt --version; then
    echo "Run pnpm install --frozen-lockfile to install the docs/config formatter." >&2
    exit 1
fi

if command -v rpp >/dev/null 2>&1; then
    rpp --version
else
    echo "Optional: install rpp to build the example pack."
fi
if [ -n "${MC_VALIDATION_ROOT:-}" ]; then
    if [ ! -x "$MC_VALIDATION_ROOT/mc-validation" ]; then
        echo "MC_VALIDATION_ROOT must point to a checkout with an executable mc-validation launcher." >&2
        exit 1
    fi
    echo "MC Validation: $MC_VALIDATION_ROOT"
else
    echo "Optional: set MC_VALIDATION_ROOT to run real-client validation."
fi
