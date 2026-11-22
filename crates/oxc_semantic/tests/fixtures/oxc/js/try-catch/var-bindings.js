function bindings(same) {
  try {
  } catch (same) {
    var same = 1;
    read(same);
  }
  read(same);

  try {
  } catch (nested) {
    try {
    } catch (nested) {
      var nested = 2;
      read(nested);
    }
    read(nested);
  }
  read(nested);

  try {
  } catch ({ message }) {
    read(message);
  }

  try {
  } catch (dynamic) {
    var dynamic = 3;
    eval("dynamic");
  }
  read(dynamic);

  try {
  } catch (shadowed) {
    with ({}) {
      var shadowed = 4;
      read(shadowed);
    }
  }
  read(shadowed);
}
