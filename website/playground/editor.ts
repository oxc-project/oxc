// Go down and find the `start` and `end` keys
export  function getStartAndEnd(view, cursor) {
  let start, end;
  while (true) {
    if (
      !start &&
      this.getTextFromView(view, cursor.from, cursor.to) == '"start"'
    ) {
      cursor.next();
      start = this.getTextFromView(view, cursor.from, cursor.to);
    }
    if (
      !end &&
      this.getTextFromView(view, cursor.from, cursor.to) == '"end"'
    ) {
      cursor.next();
      end = this.getTextFromView(view, cursor.from, cursor.to);
    }
    if (start && end) {
      break;
    }
    if (!cursor.next()) {
      break;
    }
  }

  return [start, end]
}

export const convertToUtf8 = (sourceTextUtf8, d) => {
  return new TextDecoder().decode(sourceTextUtf8.slice(0, d)).length;
}