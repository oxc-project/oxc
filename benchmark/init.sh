#!/usr/bin/env bash

mkdir -p tmp
pushd tmp
[ ! -d "vscode" ] && git clone --depth=1 git@github.com:microsoft/vscode.git
popd

npm install

rm -rf ./tmp/vscode/.eslintrc.json

cargo build --release -p oxc_cli
