export interface Order {
  id: number;
  label: string;
}

export function makeOrder(id: number, label: string): Order {
  return { id, label };
}

export function scaleOrder(order: Order, factor: number): number {
  return order.id * factor;
}

export function describeOrder(order: Order): string {
  return order.label;
}
