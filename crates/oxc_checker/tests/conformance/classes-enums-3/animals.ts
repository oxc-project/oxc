export interface Speaker {
  speak(): string;
}

export abstract class Animal implements Speaker {
  name: string;
  constructor(name: string) {
    this.name = name;
  }
  abstract speak(): string;
}

export class Dog extends Animal {
  speak(): string {
    return this.name + " woofs";
  }
}

export class Cat extends Animal {
  lives: number = 9;
  speak(): string {
    return this.name + " meows";
  }
}

export enum Mood {
  Happy = "HAPPY",
  Grumpy = "GRUMPY",
}
