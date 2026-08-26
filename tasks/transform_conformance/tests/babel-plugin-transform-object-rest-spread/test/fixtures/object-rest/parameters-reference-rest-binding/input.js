const rest = "outer";

function readAfter({ value, ...rest }, selected = value) {
  return [rest, selected];
}

function readBefore(selected = rest, { ...rest }) {
  return selected;
}
