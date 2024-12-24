let bound, bound2, bound3;

class C {
  clash1 = bound;
  clash2 = unbound;
  noClash1 = bound2;
  noClash2 = unbound2;
  noClash3 = bound3;
  noClash4 = unbound3;
  noClash5 = x => x;
  noClash6 = () => { let y; return y; };

  constructor(bound, x, y) {
    {
      var unbound;
      let bound3, unbound3;
    }
    console.log(bound, unbound, x, y);
  }
}
