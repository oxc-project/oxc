> [!NOTE]
> This is going to be a community project because I don't have the time and energy to work on this alone.

# Prettier

Background: 22.5K USD bounty for prettier written in Rust?!

See https://console.algora.io/challenges/prettier

> [!WARNING]
> ## Contribution Agreement
>
> You hereby agree that you contribute for fun and for the purpose of learning, not for the goal of winning the challenge.
>
> In the unlikely event of winning the challenge, @boshen will ultimately decide on how to spend the money.
>

> [!IMPORTANT]
Please talk to me on [discord](https://discord.com/invite/9uXCAwqQZW) and indicate that you are willing to contribute and agree to the contribution agreement.

## Getting started

Create a `test.js` and run the example `just example prettier` from `crates/oxc_prettier/examples/prettier.rs`, follow the code structure and read the references documented at the top of the files.

# Tasks

- [x] Have the basic infrastructure ready for contribution
- [ ] Implement a test runner in Rust which extracts the snapshots and do a comparison over it
- [ ] Establish a way to pass all the tests by manually porting code
- [ ] Pass as many tests as possible in https://github.com/prettier/prettier/tree/main/tests/format/js
