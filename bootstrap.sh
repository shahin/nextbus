#!/usr/bin/env sh
# Bootstrap a developer environment for this repo.

set -euo pipefail
set -x

# install pre-commit hooks
curl \
    --location \
    --remote-name \
    https://github.com/shahin/pre-commit/releases/download/v2.20.0.post1.dev1/pre-commit-2.20.0.post1.dev1.pyz
python3 pre-commit-2.20.0.post1.dev1.pyz install
rm pre-commit-2.20.0.post1.dev1.pyz
