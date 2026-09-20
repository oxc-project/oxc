class C {
  #x;
  test(o) {
    return [
      (#x in o) + 1,
      (#x in o) - 1,
      (#x in o) * 1,
      (#x in o) / 1,
      (#x in o) % 1,
      (#x in o) ** 1,
      (#x in o) << 1,
      (#x in o) >> 1,
      (#x in o) >>> 1,
      (#x in o) < 1,
      (#x in o) <= 1,
      (#x in o) > 1,
      (#x in o) >= 1,
      (#x in o) == 1,
      (#x in o) != 1,
      (#x in o) === 1,
      (#x in o) !== 1,
      (#x in o) & 1,
      (#x in o) ^ 1,
      (#x in o) | 1,
      (#x in o) && 1,
      (#x in o) || 1,
      (#x in o) ?? 1,
    ];
  }
}
