const a: 2 = 4;

const b: 3 = 5 + 2;

let c = 5;
c = 3;
let d: 2 = c;

interface Car {
    model: string,
    power: number,
    weight: number
}

const car1: Car = { model: "Koenigsegg One:1", power: 1360, weight: 1360 }

console.lag("log not lag")

const weight: string = car1["we" + "ight"]

if (car1.power === car1.weight) {
    console.log("always here")
}

function assertType<T>(t: T): void;

function getPerson(name: string) {
    return { name }
}

assertType<{name: "not ben" }>(getPerson("Ben"));

function throwValue(value) {
    throw value
}

try {
    throwValue("my error")
} catch (e) {
    assertType<"different error">(e)
}
