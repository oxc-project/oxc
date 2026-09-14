declare class A extends Base { [key]: number; }
interface B extends Outer {}
class C { declare [field]: number; abstract [method](): void; }
use(C);
