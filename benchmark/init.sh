#!/usr/bin/env bash

mkdir -p tmp
pushd tmp
[ ! -d "webpack" ] && git clone --depth=1 git@github.com:webpack/webpack.git
popd

npm install

rm -rf ./tmp/webpack/.eslintrc.js

cargo build --release -p oxc_cli
