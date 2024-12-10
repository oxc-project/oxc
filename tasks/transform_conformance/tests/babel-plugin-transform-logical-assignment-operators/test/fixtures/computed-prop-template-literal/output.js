var _unboundObj,
  _ref,
  _unboundObj2,
  _ref2;

let boundObj;

boundObj[`abc`] && (boundObj[`abc`] = 1);
(_unboundObj = unboundObj)[`abc`] && (_unboundObj[`abc`] = 2);

boundObj[_ref = `abc${foo}def`] && (boundObj[_ref] = 3);
(_unboundObj2 = unboundObj)[_ref2 = `abc${foo}def`] && (_unboundObj2[_ref2] = 4);
