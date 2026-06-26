function dec() {}

class Entity {
	@dec accessor name: string = "";
	@dec accessor count: number = 0;
	@dec accessor flag: boolean = false;
	@dec accessor untyped = "x";
	@dec accessor list: string[] = [];
	@dec static accessor sName: string = "";
	@dec accessor ["computed"]: number = 0;
}
