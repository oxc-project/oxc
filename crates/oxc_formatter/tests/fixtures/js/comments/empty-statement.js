// Comments around dropped empty statements must stay in place.
() => {
  n;
  // trailing own-line, before a dropped `;`
  ;
};
(() => {
  (p ? (() => {
    n;
    // inside conditional consequent (issue #25705)
    ;
  }) : []);
});
{
  a;
  // between statements, before a dropped `;`
  ;
  b;
}
{
  a; /* same-line */ ;
  b;
}
