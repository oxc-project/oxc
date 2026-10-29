enum Fruit {
  Apple,
  Banana,
}

const fruit: Fruit = 1;
function takesFruit(value: Fruit): void {}
takesFruit(1);
function getFruit(): Fruit {
  return 0;
}
const asserted = 1 as Fruit;
let mutable = Fruit.Apple;
mutable++;
declare const names: { [key in Fruit]: string };
names[0];

// Enum members and bitwise combinations are allowed.
const safeFruit: Fruit = Fruit.Apple;
takesFruit(Fruit.Banana);
const combined: Fruit = Fruit.Apple | Fruit.Banana;
names[Fruit.Apple];

export {};
