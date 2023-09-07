export const bar = () => import("../depth-zero").then(({foo}) => foo);
