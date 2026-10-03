function useFoo(cb = () => count++, {count} = {count: 0}) {
  return cb;
}
