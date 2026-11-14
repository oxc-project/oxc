let value = null;
cache?.set(key, (value = compute(key)));
use(value);
object?.[value = 1].method(value = 2);
callback?.(value = 3);
(object?.method)(value = 4);
outer?.method(inner?.[value = 5], value = 6);
