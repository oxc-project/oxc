// Using legacy decorators with computed member access
/// <reference lib="decorators.legacy" />
declare namespace Reflect {
	function defineMetadata(metadataKey: any, metadataValue: any, target: any, propertyKey: any): void
}
const ApiQuery = (options: any) => (
	target: Object,
	propertyKey: string | symbol,
	descriptor: PropertyDescriptor
): void => {
	Reflect.defineMetadata('api:query', options, target, propertyKey)
}
const SomeTypeMap = {
	String: String,
}

class Bar {
	@ApiQuery({
		name: 'a',
		type: SomeTypeMap.String
	})
	getFoo() {
		return 'foo'
	} 
}
