export {};

declare module "outer" {
    module "inner" {}
}

declare global {
    module "global-inner" {}
}
