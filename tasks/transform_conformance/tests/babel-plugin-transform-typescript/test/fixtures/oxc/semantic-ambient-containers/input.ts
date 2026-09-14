declare module "external" { export class C extends Base {} }
declare global { const g: number; }
use(g);
export {};
