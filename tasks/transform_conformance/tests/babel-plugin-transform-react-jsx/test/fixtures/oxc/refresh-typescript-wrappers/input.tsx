import { memo } from 'react';

function Page<T>() {
  return <div />;
}

export const As = memo(Page as any);
export const NonNull = memo(Page!);
export const Satisfies = memo(Page satisfies any);
export const Instantiation = memo(Page<any>);
export const Parenthesized = memo((Page));
export const Combined = memo(((Page<any> as any)!) satisfies any);
export const Nested = memo(memo(Page as any) satisfies any);
export const InlineArrow = memo((() => null) as any);
export const InlineFunction = memo((function () { return null; })!);
export const Unwrapped = memo(Page);
export const NotAComponent = memo(value as any);

export default memo(Page as any);
