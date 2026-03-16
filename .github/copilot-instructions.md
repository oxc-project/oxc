# Copilot Review Instructions

When reviewing pull requests in this repository:

- Treat files inside any `fixtures` directories as test data, not production code.
- Fixture files may intentionally contain buggy, unsafe, or syntactically invalid code to validate parser/linter/transformer behavior.
- Do not report normal code-quality or correctness issues for fixture files.
- You may report a fixture issue only when the test data appears incorrect or mislabeled:
  - The folder or filename does not match what the test claims to cover.
  - The fixture content does not actually test the described case.
  - The fixture appears accidentally broken rather than intentionally crafted.
