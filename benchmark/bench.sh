#!/usr/bin/env bash

TEST_DIR="./tmp/webpack/lib"

OXC="../target/release/oxc_cli lint ${TEST_DIR}"

ROME="./node_modules/rome/bin/rome check ${TEST_DIR}"

ESLINT="./node_modules/.bin/eslint -c .eslintrc.json ${TEST_DIR}"

echo ${OXC}
echo ${ROME}
echo ${ESLINT}

hyperfine -w 5 -i \
  -n oxc "${OXC}" \
  -n rome "${ROME}" \
  -n eslint "${ESLINT}"
