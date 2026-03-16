const FIELD_NAME = "myField";

function dec(target: any, key: string) {}

class MyModel {
	@dec
	[FIELD_NAME] = "value";
}
