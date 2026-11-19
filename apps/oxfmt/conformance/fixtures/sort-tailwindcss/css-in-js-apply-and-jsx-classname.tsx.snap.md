# Input

`````tsx
const A = <div className="p-4 flex" />;
const B = styled.div`
  @apply p-4 flex;
`;
const C = <div className="text-sm grid" />;
`````

# Output

## `{"sortTailwindcss":{}}`

`````tsx
const A = <div className="flex p-4" />;
const B = styled.div`
  @apply flex p-4;
`;
const C = <div className="grid text-sm" />;

`````
