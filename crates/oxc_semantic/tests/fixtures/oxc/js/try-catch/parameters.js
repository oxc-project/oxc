let a = 0, A = 0;
try { } catch (
  { a = A,
//      ^ is referenced to parent scope's `A`
    b = a }
//      ^ is referenced to parameter `a`
) {
  let A = 0 ;
}
