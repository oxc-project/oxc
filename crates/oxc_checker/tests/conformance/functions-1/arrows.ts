export const bad_arrow_expression_body = (): number => "not a number";

export const ok_arrow_expression_body = (): number => 7;

export const bad_arrow_block_body = (flag: boolean): boolean => {
  if (flag) {
    return "true";
  }
  return false;
};

export const ok_arrow_void = (count: number): void => {
  if (count > 0) {
    return;
  }
};
