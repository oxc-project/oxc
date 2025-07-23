enum Size {
  SMALL = "SMALL",
  LARGE = "LARGE",
}

enum Animal {
  CAT = "CAT",
  DOG = "DOG",
}

enum AnimalSize {
  SMALL_CAT = `${Size.SMALL}_${Animal.CAT}`,
  LARGE_DOG = `${Size.LARGE}_${Animal.DOG}`,
}
