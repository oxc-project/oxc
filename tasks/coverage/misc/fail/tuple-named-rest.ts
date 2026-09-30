type OptionalAfterRest = [...a: string[], b?: number];
type RestAfterRest = [...a: string[], ...b: number[]];
type NamedRestAfterUnnamedRest = [...string[], ...b: number[]];
type UnnamedRestAfterNamedRest = [...a: string[], ...number[]];
