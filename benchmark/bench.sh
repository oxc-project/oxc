#!/usr/bin/env bash

TEST_DIR="./tmp/vscode/src"

OXC="../target/release/oxc_cli lint ${TEST_DIR}"

ESLINT="./node_modules/.bin/eslint -c .eslintrc.json ${TEST_DIR}"

echo ${OXC}
echo ${ESLINT}

hyperfine -w 5 -i \
  -n oxc "${OXC}" \
  -n eslint "${ESLINT}"
