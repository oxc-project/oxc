let boundObj;

boundObj[true] &&= 1;
boundObj[null] &&= 2;
boundObj[123] &&= 3;
boundObj[123n] &&= 4;
boundObj[/abc/g] &&= 5;
boundObj["abc"] &&= 6;

unboundObj[true] &&= 7;
unboundObj[null] &&= 8;
unboundObj[123] &&= 9;
unboundObj[123n] &&= 10;
unboundObj[/abc/g] &&= 11;
unboundObj["abc"] &&= 12;
