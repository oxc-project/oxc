let boundObj, boundProp, mutatedObj, mutatedProp;
mutatedObj = 'x';
mutatedProp = 'x';

boundObj[boundProp] &&= 1;
boundObj[unboundProp] &&= 2;
boundObj[mutatedProp] &&= 3;

unboundObj[boundProp] &&= 4;
unboundObj[unboundProp] &&= 5;
unboundObj[mutatedProp] &&= 6;

mutatedObj[boundProp] &&= 7;
mutatedObj[unboundProp] &&= 8;
mutatedObj[mutatedProp] &&= 9;

boundObj.prop[boundProp] &&= 10;
boundObj.prop[unboundProp] &&= 11;
boundObj.prop[mutatedProp] &&= 12;

this[boundProp] &&= 13;
this[unboundProp] &&= 14;
this[mutatedProp] &&= 15;
