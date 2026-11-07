import { useCustom } from "hooks";

export function First() {
  useState(0);
  return null;
}

export const Second = () => {
  useCustom();
  return null;
};

export const Third = memo(function Third() {
  useEffect(() => {});
  return null;
});

/* Reset all components: @refresh reset */
