enum Size {
  SMALL = "tiny",
  LARGE = "big",
}

enum Animal {
  CAT = "meow",
  DOG = "woof",
}

enum AnimalSize {
  SMALL_CAT = `${Size.SMALL}_${Animal.CAT}`,
  LARGE_DOG = `${Size.LARGE}_${Animal.DOG}`,
}
