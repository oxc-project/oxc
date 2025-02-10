import A from 'mod';
namespace N1 {
  // Remove because `X` has not been referenced
  import X = A.B;
  const V = 0;
}

namespace N2 {
  // Retain because `X` has been referenced
  import X = A.B;
  const V = X;
}

