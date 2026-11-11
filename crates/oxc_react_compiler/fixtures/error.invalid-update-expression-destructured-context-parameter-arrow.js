const useFoo = (cb = () => count++, {count} = {count: 0}) => cb;
