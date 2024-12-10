let boundProp;

"abc".length &&= 1;
"abc"[boundProp] &&= 2;
"abc"[unboundProp] &&= 3;
