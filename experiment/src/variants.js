export const NO_WALK = 0;
export const WALK = 1;
export const NO_DESER = 2;

export function variantName(variant) {
  switch (variant) {
    case NO_WALK:
      return 'Without walk';
    case WALK:
      return 'With walk';
    case NO_DESER:
      return 'Without deserialization';
    default:
      throw new Error('Invalid variant');
  }
}
