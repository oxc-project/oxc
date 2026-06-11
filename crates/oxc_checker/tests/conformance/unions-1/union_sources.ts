// A union source is assignable only if every member is assignable.
type SmallNum = 1 | 2;
type BigNum = 1 | 2 | 3;

declare const small_num: SmallNum;
declare const big_num: BigNum;

const ok_subset_to_superset: BigNum = small_num;
const bad_superset_to_subset: SmallNum = big_num;

declare const str_or_num: string | number;

const ok_same_union: string | number = str_or_num;
const ok_wider_union: string | number | boolean = str_or_num;
const bad_strip_num: string = str_or_num;
const bad_strip_str: number = str_or_num;

// Single non-union source into a union target.
const ok_num_member: string | number = 42;
const bad_bool_member: string | number = true;

export {};
