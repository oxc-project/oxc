enum NumberEnum {
  NUM_1 = 1000,
  NUM_2 = 2000,
  NUM_3 = 3000,
  NUM_4 = 4000,
}

enum ComputedEnum {
  COMPUTED_1 = `${NumberEnum.NUM_1}-${NumberEnum.NUM_2}`,
  COMPUTED_2 = `${NumberEnum.NUM_3}-${NumberEnum.NUM_4}`,
}
