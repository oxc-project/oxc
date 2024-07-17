import * as React from "react";

// Both of these are equivalent:
const x = <Foo a:b="hello" />;
const y = <Foo a : b="hello" />;

interface FooProps {
    "a:b": string;
}

function Foo(props: FooProps) {
    return <div>{props["a:b"]}</div>;
}
