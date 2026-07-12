// An empty container holding a single one-line block comment stays inline (Prettier >= 3.9)
type Tup = [/* tuple */];
enum En {/* enum */}
// Note: Prettier prints an empty type literal with bracket spacing (`{ /* tl */ }`)
// but an empty object literal without (`{/* obj */}`);
// we uniformly print both without spacing
type TL = {/* tl */};

// Statement-like bodies always expand
interface I {/* iface */}
namespace N {/* ns */}
declare module "m" {/* mod */}
