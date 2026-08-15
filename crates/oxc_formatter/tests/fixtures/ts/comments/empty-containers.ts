// An empty container holding a single one-line block comment stays inline (Prettier >= 3.9)
type Tup = [/* tuple */];
enum En {/* enum */}
type TL = {/* tl */};

// Statement-like bodies always expand
interface I {/* iface */}
namespace N {/* ns */}
declare module "m" {/* mod */}
