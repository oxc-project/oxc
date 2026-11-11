# Input

`````ts
const a = clsx(`
  p-4 flex items-center justify-between ${active ? "bg-white" : "bg-black"} text-black hover:bg-gray-100
`);
const a2 = clsx.join(`
  p-4 flex items-center justify-between ${active ? "bg-white" : "bg-black"} text-black hover:bg-gray-100
`);
const b = () => tw`
  p-4 flex items-center justify-between bg-white text-black hover:bg-gray-100
`;
const c = clsx({ "a   b": someVeryLongConditionIdentifierNameThatIsReallyQuiteLongToForceBreakingHereAndMore });
const ddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd = clsx("p-4                           flex");
const eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee = clsx(`p-4                           flex`);
const f = clsx({ kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk: foo(`p-4                           flex`) });

`````

# Output

## `{"tabWidth":4,"sortTailwindcss":{"functions":["clsx","tw"]}}`

`````ts
const a = clsx(
    `flex items-center justify-between p-4 ${active ? "bg-white" : "bg-black"} text-black hover:bg-gray-100`,
);
const a2 = clsx.join(
    `flex items-center justify-between p-4 ${active ? "bg-white" : "bg-black"} text-black hover:bg-gray-100`,
);
const b = () => tw`flex items-center justify-between bg-white p-4 text-black hover:bg-gray-100`;
const c = clsx({
    "a b": someVeryLongConditionIdentifierNameThatIsReallyQuiteLongToForceBreakingHereAndMore,
});
const ddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd =
    clsx("flex p-4");
const eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee =
    clsx(`flex p-4`);
const f = clsx({
    kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk: foo(
        `p-4                           flex`,
    ),
});

`````
