declare namespace N { const value: number; }
namespace N { export const value = 1; export declare const external: number; export function read() { return external; } }
declare namespace N { function missing(): void; }
use(N);
