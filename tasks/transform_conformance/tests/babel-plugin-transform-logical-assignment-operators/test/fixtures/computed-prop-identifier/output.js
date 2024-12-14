var _unboundProp,
  _mutatedProp,
  _unboundObj,
  _unboundObj2,
  _unboundProp2,
  _unboundObj3,
  _mutatedProp2,
  _mutatedObj,
  _mutatedObj2,
  _unboundProp3,
  _mutatedObj3,
  _mutatedProp3,
  _boundObj$prop,
  _boundObj$prop2,
  _unboundProp4,
  _boundObj$prop3,
  _mutatedProp4,
  _unboundProp5,
  _mutatedProp5;

let boundObj, boundProp, mutatedObj, mutatedProp;
mutatedObj = "x";
mutatedProp = "x";

boundObj[boundProp] && (boundObj[boundProp] = 1);
boundObj[(_unboundProp = unboundProp)] && (boundObj[_unboundProp] = 2);
boundObj[(_mutatedProp = mutatedProp)] && (boundObj[_mutatedProp] = 3);

(_unboundObj = unboundObj)[boundProp] && (_unboundObj[boundProp] = 4);
(_unboundObj2 = unboundObj)[(_unboundProp2 = unboundProp)] && (_unboundObj2[_unboundProp2] = 5);
(_unboundObj3 = unboundObj)[(_mutatedProp2 = mutatedProp)] && (_unboundObj3[_mutatedProp2] = 6);

(_mutatedObj = mutatedObj)[boundProp] && (_mutatedObj[boundProp] = 7);
(_mutatedObj2 = mutatedObj)[(_unboundProp3 = unboundProp)] && (_mutatedObj2[_unboundProp3] = 8);
(_mutatedObj3 = mutatedObj)[(_mutatedProp3 = mutatedProp)] && (_mutatedObj3[_mutatedProp3] = 9);

(_boundObj$prop = boundObj.prop)[boundProp] && (_boundObj$prop[boundProp] = 10);
(_boundObj$prop2 = boundObj.prop)[(_unboundProp4 = unboundProp)] && (_boundObj$prop2[_unboundProp4] = 11);
(_boundObj$prop3 = boundObj.prop)[(_mutatedProp4 = mutatedProp)] && (_boundObj$prop3[_mutatedProp4] = 12);

this[boundProp] && (this[boundProp] = 13);
this[(_unboundProp5 = unboundProp)] && (this[_unboundProp5] = 14);
this[(_mutatedProp5 = mutatedProp)] && (this[_mutatedProp5] = 15);
