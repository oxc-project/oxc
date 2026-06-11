import { Circle, Named, Sized, Square } from "./shapes";

const ok_circle: Circle = new Circle();
const ok_named: Named = new Circle();
const ok_sized: Sized = new Circle();
const ok_square_named: Named = new Square();

const bad_square_as_circle: Circle = new Square();
const bad_circle_as_string: string = new Circle();
const bad_square_as_sized: Sized = new Square();

const ok_name: string = ok_circle.name;
const bad_radius: string = ok_circle.radius;

function area_of(sized: Sized): number {
  return sized.size();
}

const ok_area: number = area_of(ok_sized);
const bad_area_arg: number = area_of(ok_square_named);
