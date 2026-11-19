// @validateNoImpureFunctionsInRender @outputMode:"lint" @compilationMode:"infer" @panicThreshold:"none"
// Zero-argument `new Date()` is only reported in the component or hook body, not in
// callbacks, even when they are passed to an unknown function that might call them.

// The submit handler only runs on submit.
export function SubmitHandler({ form, create }) {
  const onSubmit = async (data) => {
    await create({ ...data, createdAt: new Date(), updatedAt: new Date() });
  };
  return <form onSubmit={form.handleSubmit(onSubmit)} />;
}

// Other wrappers behave the same.
export function WrappedHandler({ create }) {
  const onClick = () => create(new Date());
  return <button onClick={debounce(onClick)} />;
}

// Always-impure functions in callbacks passed to unknown functions are still reported.
export function DateNowInWrappedHandler({ create }) {
  const onClick = () => create(Date.now());
  return <button onClick={debounce(onClick)} />;
}

// `new Date()` in render is still reported.
export function NewDateInRender() {
  const now = new Date();
  return <div>{now.getTime()}</div>;
}
