## Oxlint

### Release Oxlint

- Run [Prepare Release Oxlint](https://github.com/oxc-project/oxc/actions/workflows/prepare_release_oxlint.yml)

### E2E Testing

- Run [Oxlint Ecosystem CI](https://github.com/oxc-project/oxlint-ecosystem-ci/actions/workflows/ecosystem-ci.yml)

## Oxfmt

### Release Oxfmt

- Run [Prepare Release Oxfmt](https://github.com/oxc-project/oxc/actions/workflows/prepare_release_oxfmt.yml)

## Publish Crates

- Run [Prepare Release Crates](https://github.com/oxc-project/oxc/actions/workflows/prepare_release_crates.yml)

Note: [crates.io trusted publishing](https://crates.io/docs/trusted-publishing) is configured,
a short lived token is used instead of a long-live token stored in github secrets.

## Update `VSCE_PERSONAL_ACCESS_TOKEN`

- Visit https://dev.azure.com/boshenc/_usersSettings/tokens
- Change to "Access scope: All accessible organizations"
- Edit Token
