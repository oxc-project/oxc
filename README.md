# The JavaScript Oxidation Compiler (oxc)

## Why this project?

The goal of this project is to:

* Create a blazingly fast JavaScript Compiler written in Rust.
* Provide good documentation on learning Rust and compiler techniques.

And mostly importantly, an invitation for you to come and learn Rust with me.
We will learn a lot from each other!

You can watch this project and also [follow me on twitter](https://twitter.com/boshen_c) if you don't have the time to
Rust but would like to learn things.

## Contributing

Contributions are welcome and highly appreciated. To get started, check out [CONTRIBUTING.md](./CONTRIBUTING.md).

## Call for action

We now have a fully working parser as a baseline, it is not polished yet,
so it would be much appreciated if I can invite you and review any of the code and point out for improvements.
I welcome all nitpickings and bikesheddings.

I have also created some [discussions](https://github.com/Boshen/oxc/discussions) for documenting my thought processes.

## Milestone

The current objective is to improve the parser for real usage. Areas include:

* API
* Diagnostics reporting
* Performance
* Pass more conformance tests

You may start with https://github.com/Boshen/oxc/issues/36


## Conformance

The `cargo coverage` command currently reports the following summary

```
Test262 Summary:
AST Parsed     : 43934/43934 (100.00%)

Babel Summary:
AST Parsed     : 2045/2057 (99.42%)

TypeScript Summary:
AST Parsed     : 4291/4861 (88.27%)
```

(The parser is failing some of the TypeScript recoverable parser tests.)

## Learning Resources

* My [small tutorial on how to write a JavaScript Parser in Rust](https://boshen.github.io/javascript-parser-in-rust/)
* [Insert your inspirational learning resources here]

## Credits

This project is inspired by the following great mentors and projects:

* [Rome Tools](https://rome.tools) - [@MichaReiser](https://github.com/MichaReiser), [@ematipico](https://github.com/ematipico)
* [Ruff](https://beta.ruff.rs) - [@charliermarsh](https://github.com/charliermarsh)
* [quick-lint-js](https://quick-lint-js.com) - [@strager](https://github.com/strager)

## License

[MIT](./LICENSE)
