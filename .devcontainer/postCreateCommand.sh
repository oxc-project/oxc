#!/bin/bash
set -eu

sudo chown -R "$(whoami)" "$(pwd)/target"

# cargo binstall (NOTE: update the commit when the script changes)
curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/3b64237893f03d6cce15ff4c5163026b681d5f9e/install-from-binstall-release.sh | bash

npm i -g @withgraphite/graphite-cli

cargo binstall ast-grep -y

just init
