export interface Rect {
  width: number;
  height: number;
}

export function area(rect: Rect): number {
  return rect.width * rect.height;
}

export function tag(rect: Rect, name: string): string {
  return name + ":" + rect.width + "x" + rect.height;
}

export const unit_rect: Rect = { width: 1, height: 1 };
