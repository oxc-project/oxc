# Input

`````tsx
<div className={clsx('text-md before:content-["hello"]')}>Hello</div>;
<div className={clsx("text-md before:content-['hello']")}>Hello</div>;
<div className={showLandingPage ? "container pb-6" : 'hidden'}>title</div>

`````

# Output

## `{"sortTailwindcss":{"functions":["clsx"]},"singleQuote":true}`

`````tsx
<div className={clsx('text-md before:content-["hello"]')}>Hello</div>;
<div className={clsx("text-md before:content-['hello']")}>Hello</div>;
<div className={showLandingPage ? 'container pb-6' : 'hidden'}>title</div>;

`````

## `{"sortTailwindcss":{"functions":["clsx"]}}`

`````tsx
<div className={clsx('text-md before:content-["hello"]')}>Hello</div>;
<div className={clsx("text-md before:content-['hello']")}>Hello</div>;
<div className={showLandingPage ? "container pb-6" : "hidden"}>title</div>;

`````

## `{"sortTailwindcss":{},"jsxSingleQuote":true}`

`````tsx
<div className={clsx('text-md before:content-["hello"]')}>Hello</div>;
<div className={clsx("text-md before:content-['hello']")}>Hello</div>;
<div className={showLandingPage ? "container pb-6" : "hidden"}>title</div>;

`````
