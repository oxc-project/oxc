import Child from "./Child";
import Other from "./Other";
import { ButtonClickEvent, EventChild } from "./events";
import myModule from "./MyModule";

// Allowed by `allow`.
export function AllowedName() {
  return <div>{Child()}</div>;
}

// Allowed by `allowPattern`.
export function AllowedPattern() {
  const event = ButtonClickEvent();
  return <button onClick={() => console.log(event)} />;
}

// Not in `allow` and does not match `allowPattern`.
export function NotAllowed() {
  return <div>{Other()}</div>;
}

// `allowPattern` is a regex, so `Event$` only matches names ending in `Event`.
export function PatternDoesNotMatch() {
  return <div>{EventChild()}</div>;
}

// Like the compiler's `validateNoCapitalizedCalls`, the allowlist only applies to global loads.
export function MethodCall() {
  return <div>{myModule.Child()}</div>;
}
