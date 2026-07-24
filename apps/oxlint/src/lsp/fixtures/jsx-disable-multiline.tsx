export function Redirect() {
  return (
    <>
      <p>Another child to justify fragment</p>
      <div
        onClick={() => history.redirect("/")}
        role="link"
      />
    </>
  );
}
