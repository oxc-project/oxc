export const enabled_plain = true;
export let enabled_plain_let = true;
export var enabled_plain_var = true;
export const enabled_plain_annotated: boolean = true;
export const enabled_parenthesized = (true);
export let enabled_parenthesized_let = (true);
export var enabled_parenthesized_var = (true);
export const enabled_parenthesized_annotated: boolean = (true);
export const enabled_nested = (((true)));
export let enabled_nested_let = (((true)));
export var enabled_nested_var = (((true)));
export const enabled_nested_annotated: boolean = (((true)));

export const disabled_plain = false;
export let disabled_plain_let = false;
export var disabled_plain_var = false;
export const disabled_plain_annotated: boolean = false;
export const disabled_parenthesized = (false);
export let disabled_parenthesized_let = (false);
export var disabled_parenthesized_var = (false);
export const disabled_parenthesized_annotated: boolean = (false);
export const disabled_nested = (((false)));
export let disabled_nested_let = (((false)));
export var disabled_nested_var = (((false)));
export const disabled_nested_annotated: boolean = (((false)));

export const number_plain = 1;
export let number_plain_let = 1;
export var number_plain_var = 1;
export const number_plain_annotated: number = 1;
export const number_parenthesized = (1);
export let number_parenthesized_let = (1);
export var number_parenthesized_var = (1);
export const number_parenthesized_annotated: number = (1);
export const number_nested = (((1)));
export let number_nested_let = (((1)));
export var number_nested_var = (((1)));
export const number_nested_annotated: number = (((1)));

export const negative_plain = -1;
export let negative_plain_let = -1;
export var negative_plain_var = -1;
export const negative_plain_annotated: number = -1;
export const negative_parenthesized = (-1);
export let negative_parenthesized_let = (-1);
export var negative_parenthesized_var = (-1);
export const negative_parenthesized_annotated: number = (-1);
export const negative_nested = (((-1)));
export let negative_nested_let = (((-1)));
export var negative_nested_var = (((-1)));
export const negative_nested_annotated: number = (((-1)));

export const bigint_plain = 1n;
export let bigint_plain_let = 1n;
export var bigint_plain_var = 1n;
export const bigint_plain_annotated: bigint = 1n;
export const bigint_parenthesized = (1n);
export let bigint_parenthesized_let = (1n);
export var bigint_parenthesized_var = (1n);
export const bigint_parenthesized_annotated: bigint = (1n);
export const bigint_nested = (((1n)));
export let bigint_nested_let = (((1n)));
export var bigint_nested_var = (((1n)));
export const bigint_nested_annotated: bigint = (((1n)));

export const string_plain = "";
export let string_plain_let = "";
export var string_plain_var = "";
export const string_plain_annotated: string = "";
export const string_parenthesized = ("");
export let string_parenthesized_let = ("");
export var string_parenthesized_var = ("");
export const string_parenthesized_annotated: string = ("");
export const string_nested = ((("")));
export let string_nested_let = ((("")));
export var string_nested_var = ((("")));
export const string_nested_annotated: string = ((("")));

export const template_plain = `text`;
export let template_plain_let = `text`;
export var template_plain_var = `text`;
export const template_plain_annotated: string = `text`;
export const template_parenthesized = (`text`);
export let template_parenthesized_let = (`text`);
export var template_parenthesized_var = (`text`);
export const template_parenthesized_annotated: string = (`text`);
export const template_nested = (((`text`)));
export let template_nested_let = (((`text`)));
export var template_nested_var = (((`text`)));
export const template_nested_annotated: string = (((`text`)));

const separate_true = true;
const separate_false = false;
const separate_parenthesized = (((true)));
export { separate_true, separate_false, separate_parenthesized };

export const asserted = (true as const);
export const object = ({ x: true });
export const nil = (null);

export const positive_plain = +1;
export const positive_parenthesized = (+1);
export const positive_nested = (((+1)));
export let positive_mutable = (+1);
export const positive_annotated: number = (+1);
export class Positive {
    readonly plain = +1;
    readonly parenthesized = (+1);
    readonly negative = (-1);
}
