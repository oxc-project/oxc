// @outputMode:"lint" @validateNoImpureFunctionsInRender
// Deferred callbacks may create timestamps, while render-time callbacks remain impure.
export function SubmitForm(form) {
  const onSubmit = async (formData) => {
    await create({...formData, createdAt: new Date(), updatedAt: new Date()});
    Date.now();
  };
  return <form onSubmit={form.handleSubmit(onSubmit)} />;
}
export function DirectHandler() {
  return <button onClick={() => Math.random()} />;
}
export function DeclaredHandler() {
  function onSubmit() { new Date(); Date.now(); }
  return <form onSubmit={onSubmit} />;
}
export function DeferredIteratorHandler({items}) {
  function handlePick(item) { console.log(item, Date.now()); }
  return <div>{items.map(item => <button key={item} onClick={() => handlePick(item)} />)}</div>;
}
export function RenderMap(items) { return <div>{items.map(() => Date.now())}</div>; }
export function RenderFilter(items) { return <div>{items.filter(() => new Date()).length}</div>; }
export function NestedRenderMap(items) {
  const getTime = () => new Date();
  return <div>{items.map(() => getTime())}</div>;
}
export function DirectCall() {
  const getTime = () => Date.now();
  return <div>{getTime()}</div>;
}
