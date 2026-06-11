// Clean-only fixture: every assignment here is accepted by tsc.
export interface Ok {
  tag: "ok";
  value: number;
}
export interface Err {
  tag: "err";
  message: string;
}
export type Result = Ok | Err;

export function describe_result(r: Result): string {
  return r.tag;
}
