const navTitle = getTitle();

function NavTitle() {
  const Title = navTitle;
  return <Title value={1} />;
}

export const FIXTURE_ENTRYPOINT = {
  fn: NavTitle,
  params: [],
};
