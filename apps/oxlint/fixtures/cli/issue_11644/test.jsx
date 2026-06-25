// oxlint-disable-next-line no-unused-vars
function Foo() {
  return (
    <template>
      <label>
        <span>name</span>
        <input
          defaultValue={
            actionState.payload?.get("name")?.toString() ?? "Pool party"
          }
          name="name"
          placeholder="name"
          type="text"
        />
      </label>
    </template>
  )
}
