const FIELD_NAME = "myField";

const decoratedKeys: string[] = [];

function Field() {
  return (target: any, key: string) => {
    decoratedKeys.push(key);
  };
}

class MyModel {
  @Field()
  [FIELD_NAME]: string;
}

expect(decoratedKeys).toEqual(["myField"]);

// Ensure the property is not defined on instances (removeClassFieldsWithoutInitializer: true)
const instance = new MyModel();
expect(Object.prototype.hasOwnProperty.call(instance, "myField")).toBe(false);
