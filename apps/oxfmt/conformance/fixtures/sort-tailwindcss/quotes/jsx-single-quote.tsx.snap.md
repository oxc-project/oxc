# Input

`````tsx
<div className="text-md before:content-['hello']">Hello</div>;
<div className='text-md before:content-["hello"]'>Hello</div>;

`````

# Output

## `{"sortTailwindcss":{"functions":["clsx"]},"singleQuote":true}`

`````tsx
<div className="text-md before:content-['hello']">Hello</div>;
<div className='text-md before:content-["hello"]'>Hello</div>;

`````

## `{"sortTailwindcss":{"functions":["clsx"]}}`

`````tsx
<div className="text-md before:content-['hello']">Hello</div>;
<div className='text-md before:content-["hello"]'>Hello</div>;

`````

## `{"sortTailwindcss":{},"jsxSingleQuote":true}`

`````tsx
<div className="text-md before:content-['hello']">Hello</div>;
<div className='text-md before:content-["hello"]'>Hello</div>;

`````
