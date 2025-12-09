#! /usr/bin/env bash

set -euo pipefail

dir="$(dirname "$0")/data"
rm -rf "${dir}"
mkdir -p "${dir}"
cd "${dir}"

curl -fsSLo small.json 'https://api.github.com/repos/callum-oakley/json5-rs/commits?per_page=1'
curl -fsSLo medium.json 'https://api.github.com/repos/callum-oakley/json5-rs/commits?per_page=10'
curl -fsSLo large.json 'https://api.github.com/repos/callum-oakley/json5-rs/commits?per_page=100'
