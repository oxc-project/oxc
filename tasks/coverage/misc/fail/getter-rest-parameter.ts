class C { get x(...args) {} }
({ get x(...args) {} });
interface I { get x(...args: any[]): string }
type T = { get x(...args: any[]): string };
