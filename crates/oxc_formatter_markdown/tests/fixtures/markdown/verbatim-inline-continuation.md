<!-- Continuation lines of verbatim inline nodes resume at the container column (`- `, not the printed alignment), whitespace after it kept: it is content -->

- [ ] a `x
      y` b

- [ ] **CI is green** - <!-- if any check is red or skipped, name it and say why. "Deploy" and
      "Tear down PR preview" showing skipped is expected. -->

- [ ] a $x
      y$ b

- [ ] ![alt
      text](u) and [t](u "one
      two")

- Top-level:
  - Sub-bullet wraps and contains
    `someInlineCode(arg1, arg2,
    arg3, arg4)` and continues with more text after the
    code span closes.
