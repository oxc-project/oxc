import { area, tag, unit_rect, type Rect } from "./shapes";

// Clean-only fixture: every access and call matches the annotations.
const ok_rect: Rect = { width: 3, height: 4 };
const ok_area: number = area(ok_rect);
const ok_unit_area: number = area(unit_rect);
const ok_width: number = unit_rect.width;
const ok_height: number = ok_rect.height;
const ok_tag: string = tag(ok_rect, "box");
const ok_inline_tag: string = tag({ width: 5, height: 6 }, "inline");

export const exported_area_sum: number =
  ok_area + ok_unit_area + ok_width + ok_height;
export const exported_tags: string = ok_tag + ok_inline_tag;
