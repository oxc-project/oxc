export const getter = {
  get value(): number {
    return 1;
  },
};

export const setter = {
  set value(v: number) {},
};

export const pair = {
  get value(): number {
    return 1;
  },
  set value(v: number) {},
};

export const constGetter = {
  get value(): number {
    return 1;
  },
} as const;

export const constPair = {
  get value(): number {
    return 1;
  },
  set value(v: number) {},
} as const;
