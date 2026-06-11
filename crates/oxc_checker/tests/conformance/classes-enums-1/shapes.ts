export interface Named {
  name: string;
}

export interface Sized {
  size(): number;
}

export class Circle implements Named, Sized {
  name: string = "circle";
  radius: number = 1;
  size(): number {
    return this.radius * 2;
  }
}

export class Square implements Named {
  name: string = "square";
  side: number = 2;
}

export class BadShape implements Sized {
  name: string = "bad";
}

export class WrongName implements Named {
  name: number = 5;
}
