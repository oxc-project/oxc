# Input

`````js
// An escaped newline in a string holding `${}` breaks no selector
const a = styled.div`
  div span[foo="a long long long long long long long long\
long long ${x} long long attribute value"] {
  }
`;

// A `${}` only inside a string breaks no selector list, a string stays as is
const b = styled.div`
  a  b, div  span[foo="a  ${x}"], c  d {
  }
  p  ${y}, div  span[foo="a  b\
    c"] {
  }
`;

`````

# Output

## `{"printWidth":80}`

`````js
// An escaped newline in a string holding `${}` breaks no selector
const a = styled.div`
  div span[foo="a long long long long long long long long\
long long ${x} long long attribute value"] {
  }
`;

// A `${}` only inside a string breaks no selector list, a string stays as is
const b = styled.div`
  a b,
  div span[foo="a  ${x}"],
  c d {
  }
  p ${y}, div span[foo="a  b\
    c"] {
  }
`;

`````

## `{"printWidth":100}`

`````js
// An escaped newline in a string holding `${}` breaks no selector
const a = styled.div`
  div span[foo="a long long long long long long long long\
long long ${x} long long attribute value"] {
  }
`;

// A `${}` only inside a string breaks no selector list, a string stays as is
const b = styled.div`
  a b,
  div span[foo="a  ${x}"],
  c d {
  }
  p ${y}, div span[foo="a  b\
    c"] {
  }
`;

`````
