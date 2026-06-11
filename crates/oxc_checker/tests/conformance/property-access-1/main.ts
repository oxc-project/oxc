import { describeOrder, makeOrder, scaleOrder, type Order } from "./lib";

// Clean calls: argument types match the imported annotations.
const ok_order: Order = makeOrder(1, "first");
const ok_scaled: number = scaleOrder(ok_order, 2);
const ok_label: string = describeOrder(ok_order);
const ok_inline: string = describeOrder({ id: 4, label: "inline" });

// Wrong argument types to functions imported from ./lib.
const bad_id = makeOrder("not-a-number", "second");
const bad_factor = scaleOrder(ok_order, "twice");
const bad_shape = describeOrder({ id: 3 });
const bad_both = makeOrder(true, 42);

// Result types still flow through despite bad arguments.
const ok_followup: string = ok_label + ok_inline;
export const exported_total: number = ok_scaled;
export const exported_text: string = ok_followup;
