#!/usr/bin/env sh
# Bootstrap a developer environment for this repo.

set -euo pipefail
set -x

version="2.20.0.post1.dev3"

# install pre-commit hooks
curl \
    --location \
    --remote-name \
    https://github.com/shahin/pre-commit/releases/download/v${version}/pre-commit-${version}.pyz
python3 pre-commit-${version}.pyz install
rm pre-commit-${version}.pyz
