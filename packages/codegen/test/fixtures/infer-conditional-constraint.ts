type C1<T> = T extends infer U extends (true extends true ? string : number) ? U : never;
type C2<T> = T extends [infer U extends (string extends T ? 1 : 2)] ? U : never;
type C3<T> = T extends Array<infer U extends (string extends T ? 1 : 2)> ? U : never;

type N1<T> = T extends [infer U] ? U : never;
type N2<T> = T extends [infer U extends (string)] ? U : never;
type N3<T> = T extends [infer U extends () => (string extends T ? 1 : 2)] ? U : never;
type N4<T> = T extends [infer U extends keyof (string extends T ? 1 : 2)] ? U : never;
type N5<T> = T extends [(infer U extends string) extends T ? 1 : 2] ? U : never;
