// @validateNoImpureFunctionsInRender @outputMode:"lint" @compilationMode:"infer" @panicThreshold:"none"
const ITEMS = [{ id: "a" }, { id: "b" }];

// The handler's impure call only runs after a click, not while rendering the list.
export function HelperCalledFromMapHandler({ onPick }) {
  function handlePick(item) {
    onPick(item.id, Date.now());
  }
  return (
    <div>
      {ITEMS.map((item) => (
        <button key={item.id} onClick={() => handlePick(item)} />
      ))}
    </div>
  );
}

// An inline event handler may itself call an impure function.
export function InlineHandlerInsideMap() {
  return (
    <div>
      {ITEMS.map((item) => (
        <button key={item.id} onClick={() => Date.now()} />
      ))}
    </div>
  );
}

// Passing the handler directly has the same event-time behavior.
export function HandlerPassedDirectlyInsideMap() {
  function handlePick() {
    Date.now();
  }
  return (
    <div>
      {ITEMS.map((item) => (
        <button key={item.id} onClick={handlePick} />
      ))}
    </div>
  );
}

// The same helper is safe when its event wrapper is outside the map callback.
export function HelperCalledOutsideMap({ onPick }) {
  function handlePick(item) {
    onPick(item.id, Date.now());
  }
  return (
    <div>
      <button onClick={() => handlePick(ITEMS[0])} />
    </div>
  );
}

// Calls from the map callback itself still execute during render.
export function ImpureCallInMapCallback() {
  return <div>{ITEMS.map((item) => Date.now())}</div>;
}

// A helper called directly from the map callback also runs during render.
export function HelperCalledFromMapCallback() {
  function getTime() {
    return Date.now();
  }
  return <div>{ITEMS.map((item) => getTime())}</div>;
}
