export class C {
  readonly instanceBare = "i";
  readonly instanceConst = "ic" as const;
  static readonly staticBare = "s";
  static readonly staticConst = "sc" as const;
  static readonly A = "a" as const;
  static readonly B: "b" = "b";
  static D = "d" as const;
}

export const bare = "e" as const;
