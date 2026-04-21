enum NumberEnum {
  A = 1,
  B = 2,
}

enum ComputedEnum {
  C = `prefix-${NumberEnum.A}-middle-${NumberEnum.B}-suffix`,
  D = `${NumberEnum.A}-suffix`,
}
