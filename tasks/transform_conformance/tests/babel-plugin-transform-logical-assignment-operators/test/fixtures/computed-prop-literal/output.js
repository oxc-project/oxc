var _unboundObj,
  _unboundObj2,
  _unboundObj3,
  _unboundObj4,
  _unboundObj5,
  _unboundObj6;

let boundObj;

boundObj[true] && (boundObj[true] = 1);
boundObj[null] && (boundObj[null] = 2);
boundObj[123] && (boundObj[123] = 3);
boundObj[123n] && (boundObj[123n] = 4);
boundObj[/abc/g] && (boundObj[/abc/g] = 5);
boundObj["abc"] && (boundObj["abc"] = 6);

(_unboundObj = unboundObj)[true] && (_unboundObj[true] = 7);
(_unboundObj2 = unboundObj)[null] && (_unboundObj2[null] = 8);
(_unboundObj3 = unboundObj)[123] && (_unboundObj3[123] = 9);
(_unboundObj4 = unboundObj)[123n] && (_unboundObj4[123n] = 10);
(_unboundObj5 = unboundObj)[/abc/g] && (_unboundObj5[/abc/g] = 11);
(_unboundObj6 = unboundObj)["abc"] && (_unboundObj6["abc"] = 12);
