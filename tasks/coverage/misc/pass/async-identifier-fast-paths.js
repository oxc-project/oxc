const plain = async foo => ({ foo, "key": foo });
const contextual = async get => ({ get, "key": get });
const escaped = async \u0066oo => ({ foo, "key": \u0066oo });
