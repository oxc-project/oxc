function noOuterBinding() {
  try { throw 0; } catch (y) {
    var y = 2;
    use(y);
  }
  use(y);
}

function parameter(y) {
  try { throw 0; } catch (y) {
    var y = 2;
    use(y);
  }
  use(y);
}

function priorVar() {
  var y = 1;
  try { throw 0; } catch (y) {
    var y = 2;
    use(y);
  }
  use(y);
}

function laterVar() {
  try { throw 0; } catch (y) {
    var y = 2;
    use(y);
  }
  var y = 1;
  use(y);
}

function nested() {
  try { throw 10; } catch (y) {
    try { throw 20; } catch (y) {
      var y = 3;
      use(y);
    }
    use(y);
  }
  use(y);
}

function nestedBlock() {
  try { throw 0; } catch (y) {
    {
      var y = 1;
      var y = 2;
      use(y);
    }
  }
  use(y);
}

function separateCatches() {
  try { throw 1; } catch (y) { var y = 2; use(y); }
  try { throw 3; } catch (y) { var y = 4; use(y); }
  use(y);
}
