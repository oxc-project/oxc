const patterns = [
  /[]/,
  /[a-z\d]/,
  /[a-z\p{Letter}]/u,
  /[a-z]/v,
  /[\q{}]/v,
  /[\q{a|b}]/v,
  /[\q{ab}]/v,
  /[\p{Basic_Emoji}]/v,
  /[[\q{ab}]&&[a-z]]/v,
  /[[\q{ab}]--[a-z]]/v,
  /[[a-z]--[\q{ab}]]/v,
];
