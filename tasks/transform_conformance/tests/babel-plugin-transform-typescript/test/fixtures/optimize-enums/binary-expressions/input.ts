enum Flags {
  Read = 1 << 0,
  Write = 1 << 1,
  Execute = 1 << 2,
  ReadWrite = Read | Write,
}

Flags.Read;
Flags.ReadWrite;
