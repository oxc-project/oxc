enum LocalFlag {
  Off = 0,
  On = 1,
}

const enum LocalMode {
  Fast = "FAST",
  Slow = "SLOW",
}

const ok_flag_as_number: number = LocalFlag.On;
const ok_mode_as_string: string = LocalMode.Fast;
const bad_flag_as_string: string = LocalFlag.On;
const bad_mode_as_number: number = LocalMode.Slow;

export {};
