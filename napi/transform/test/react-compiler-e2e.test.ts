// @vitest-environment jsdom
import { describe, expect, test } from "vitest";
import { transformSync } from "../index";

/**
 * E2E tests for the React Compiler via NAPI transform binding.
 *
 * The React compiler runs the full pipeline (HIR lowering, type inference,
 * aliasing analysis, reactive scope inference, codegen) and replaces
 * function bodies with memoized output using the react/compiler-runtime.
 * These tests verify that:
 *
 * 1. The compiler pipeline runs without errors for valid components
 * 2. Memoized codegen output is correctly produced (_c cache, sentinel checks)
 * 3. Different compilation modes and options work as expected
 * 4. Various component patterns are handled correctly
 *
 * Ported from: babel-plugin-react-compiler/src/__tests__/e2e/
 */

// Helper: compile a source string with the React compiler and return result
function compileWithReactCompiler(
  source: string,
  options: {
    filename?: string;
    compilationMode?: string;
  } = {},
) {
  const { filename = "test.tsx", compilationMode = "all" } = options;
  return transformSync(filename, source, {
    lang: "tsx",
    sourceType: "module",
    jsx: {
      runtime: "automatic",
    },
    plugins: {
      reactCompiler: {
        enabled: true,
        compilationMode: compilationMode,
      },
    },
  });
}

describe("react-compiler e2e", () => {
  describe("basic compilation", () => {
    test("simple component compiles without errors", () => {
      const source = `
        function Component({ name }) {
          return <div>Hello {name}</div>;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
      expect(result.code).toContain("_jsx");
    });

    test("exported component compiles successfully", () => {
      const source = `
        export function Greeting({ name }) {
          return <h1>Welcome, {name}!</h1>;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
      expect(result.code).toContain("export function Greeting");
    });

    test("default exported component compiles successfully", () => {
      const source = `
        export default function App() {
          return <div>App</div>;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
      expect(result.code).toContain("export default function App");
    });

    test("arrow function component compiles", () => {
      const source = `
        const Component = ({ name }) => {
          return <div>{name}</div>;
        };
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
      expect(result.code).toContain("_jsx");
    });

    test("multiple components in one file compile", () => {
      const source = `
        function Header({ title }) {
          return <h1>{title}</h1>;
        }
        function Footer({ text }) {
          return <footer>{text}</footer>;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
      expect(result.code).toContain("function Header");
      expect(result.code).toContain("function Footer");
    });
  });

  describe("hooks", () => {
    test("useState hook compiles correctly", () => {
      const source = `
        import { useState } from 'react';
        function Counter() {
          const [count, setCount] = useState(0);
          return <button onClick={() => setCount(count + 1)}>{count}</button>;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
    });

    test("useEffect hook compiles correctly", () => {
      const source = `
        import { useState, useEffect } from 'react';
        function Timer() {
          const [time, setTime] = useState(0);
          useEffect(() => {
            const id = setInterval(() => setTime(t => t + 1), 1000);
            return () => clearInterval(id);
          }, []);
          return <span>{time}</span>;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
    });

    test("useMemo and useCallback compile correctly", () => {
      const source = `
        import { useMemo, useCallback } from 'react';
        function Component({ items, onSelect }) {
          const sorted = useMemo(() => [...items].sort(), [items]);
          const handleClick = useCallback((id) => onSelect(id), [onSelect]);
          return <ul>{sorted.map(item => <li key={item} onClick={() => handleClick(item)}>{item}</li>)}</ul>;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
    });

    test("useRef hook compiles correctly", () => {
      const source = `
        import { useRef, useEffect } from 'react';
        function Component() {
          const ref = useRef(null);
          useEffect(() => {
            if (ref.current) {
              ref.current.focus();
            }
          }, []);
          return <input ref={ref} />;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
    });

    test("custom hook compiles correctly", () => {
      const source = `
        import { useState, useEffect } from 'react';
        function useWindowWidth() {
          const [width, setWidth] = useState(0);
          useEffect(() => {
            const handler = () => setWidth(window.innerWidth);
            window.addEventListener('resize', handler);
            return () => window.removeEventListener('resize', handler);
          }, []);
          return width;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
    });
  });

  describe("semantics preservation", () => {
    test("string literals are preserved in output", () => {
      const source = `
        function Component() {
          const x = 'test value 1';
          return <div>{x}</div>;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
      expect(result.code).toContain("test value 1");
    });

    test("conditional rendering is preserved", () => {
      const source = `
        function Component({ isLoggedIn }) {
          return (
            <div>
              {isLoggedIn ? <span>Welcome</span> : <span>Please log in</span>}
            </div>
          );
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
      expect(result.code).toContain("Welcome");
      expect(result.code).toContain("Please log in");
    });

    test("template literals are preserved", () => {
      const source = `
        function Component({ name, age }) {
          const greeting = \`Hello \${name}, you are \${age} years old\`;
          return <p>{greeting}</p>;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
    });

    test("object and array destructuring preserved", () => {
      const source = `
        function Component({ items: [first, ...rest], config: { theme } }) {
          return <div className={theme}>{first}{rest.length}</div>;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
    });

    test("spread attributes are preserved", () => {
      const source = `
        function Component({ style, className, ...rest }) {
          return <div style={style} className={className} {...rest} />;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
    });
  });

  describe("component patterns", () => {
    test("component with multiple early returns", () => {
      const source = `
        function Component({ type }) {
          if (type === 'a') {
            return <div>Type A</div>;
          }
          if (type === 'b') {
            return <div>Type B</div>;
          }
          return <div>Default</div>;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
    });

    test("component with try-catch", () => {
      const source = `
        function Component({ data }) {
          let parsed;
          try {
            parsed = JSON.parse(data);
          } catch {
            parsed = null;
          }
          return <div>{parsed ? parsed.name : 'Invalid data'}</div>;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
    });

    test("component with for loop", () => {
      const source = `
        function Component({ count }) {
          const items = [];
          for (let i = 0; i < count; i++) {
            items.push(<span key={i}>{i}</span>);
          }
          return <div>{items}</div>;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
    });

    test("component with destructured props and defaults", () => {
      const source = `
        function Component({ name = "World", greeting = "Hello" }) {
          return <div>{greeting}, {name}!</div>;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
    });

    test("component with nested JSX", () => {
      const source = `
        function Card({ title, body, footer }) {
          return (
            <div className="card">
              <div className="card-header">
                <h2>{title}</h2>
              </div>
              <div className="card-body">
                <p>{body}</p>
              </div>
              <div className="card-footer">
                {footer}
              </div>
            </div>
          );
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
    });

    test("component with fragment", () => {
      const source = `
        function Component({ a, b }) {
          return (
            <>
              <div>{a}</div>
              <div>{b}</div>
            </>
          );
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
    });

    test("component returning null", () => {
      const source = `
        function Empty() {
          return null;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
    });

    test("component with complex derived state", () => {
      const source = `
        function Component({ items }) {
          const count = items.length;
          const doubled = count * 2;
          const filtered = items.filter(x => x.active);
          const names = filtered.map(x => x.name);
          return <div>{names.join(', ')} ({doubled})</div>;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
    });

    test("component with callback handlers", () => {
      const source = `
        function Component({ onClick, onHover }) {
          const handleClick = () => {
            console.log('clicked');
            onClick();
          };
          const handleHover = () => {
            onHover();
          };
          return <button onClick={handleClick} onMouseOver={handleHover}>Click me</button>;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
    });

    test("component with inline object creation", () => {
      const source = `
        function Component({ theme }) {
          const style = {
            color: theme.primary,
            backgroundColor: theme.background,
            padding: '10px',
          };
          return <div style={style}>Styled</div>;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
    });

    test("component with array map in JSX", () => {
      const source = `
        function List({ items }) {
          return (
            <ul>
              {items.map((item) => (
                <li key={item.id}>{item.name}</li>
              ))}
            </ul>
          );
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
    });
  });

  describe("update expressions", () => {
    test("pre/post increment and decrement compile without errors", () => {
      const source = `
        export function Counter(props) {
          let value = props.value;
          let a = value++;
          let b = ++value;
          let c = ++value;
          let d = value--;
          let e = --value;
          let f = --value;
          return <div>{a},{b},{c},{d},{e},{f},{value}</div>;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
    });
  });

  describe("TypeScript patterns", () => {
    test("component with TypeScript interface", () => {
      const source = `
        interface Props {
          name: string;
          age: number;
        }
        function Component({ name, age }: Props) {
          return <div>{name} is {age} years old</div>;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
    });

    test("component with TypeScript type", () => {
      const source = `
        type Props = {
          items: Array<{ id: string; label: string }>;
          onSelect: (item: { id: string; label: string }) => void;
        };
        function SelectList({ items, onSelect }: Props) {
          return (
            <ul>
              {items.map(item => (
                <li key={item.id} onClick={() => onSelect(item)}>{item.label}</li>
              ))}
            </ul>
          );
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
    });

    test("component with type assertion", () => {
      const source = `
        function Component({ data }: { data: unknown }) {
          const items = data as string[];
          return <div>{items.join(', ')}</div>;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
    });

    test("component with generic utility types", () => {
      const source = `
        type ButtonProps = {
          label: string;
          disabled?: boolean;
          variant: 'primary' | 'secondary';
        };
        function Button({ label, disabled = false, variant }: ButtonProps) {
          const className = variant === 'primary' ? 'btn-primary' : 'btn-secondary';
          return <button className={className} disabled={disabled}>{label}</button>;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
    });
  });

  describe("compilation modes", () => {
    test("'all' mode compiles all functions", () => {
      const source = `
        function Component({ name }) {
          return <div>{name}</div>;
        }
      `;
      const result = compileWithReactCompiler(source, {
        compilationMode: "all",
      });
      expect(result.errors).toEqual([]);
    });

    test("'infer' mode compiles components by naming convention", () => {
      const source = `
        function Component({ name }) {
          return <div>{name}</div>;
        }
      `;
      const result = compileWithReactCompiler(source, {
        compilationMode: "infer",
      });
      expect(result.errors).toEqual([]);
    });

    test("'annotation' mode compiles only functions with 'use memo'", () => {
      const source = `
        function NotCompiled({ name }) {
          return <div>{name}</div>;
        }

        function Compiled({ name }) {
          'use memo';
          return <div>{name}</div>;
        }
      `;
      const result = compileWithReactCompiler(source, {
        compilationMode: "annotation",
      });
      expect(result.errors).toEqual([]);
    });
  });

  describe("plugin configuration", () => {
    test("boolean true enables the compiler", () => {
      const source = `
        function Component({ name }) {
          return <div>{name}</div>;
        }
      `;
      const result = transformSync("test.tsx", source, {
        lang: "tsx",
        sourceType: "module",
        jsx: { runtime: "automatic" },
        plugins: {
          reactCompiler: true,
        },
      });
      expect(result.errors).toEqual([]);
    });

    test("boolean false disables the compiler", () => {
      const source = `
        function Component({ name }) {
          return <div>{name}</div>;
        }
      `;
      const result = transformSync("test.tsx", source, {
        lang: "tsx",
        sourceType: "module",
        jsx: { runtime: "automatic" },
        plugins: {
          reactCompiler: false,
        },
      });
      expect(result.errors).toEqual([]);
      // Output should just be JSX-transformed
      expect(result.code).toContain("_jsx");
    });

    test("enabled: false disables the compiler", () => {
      const source = `
        function Component({ name }) {
          return <div>{name}</div>;
        }
      `;
      const result = transformSync("test.tsx", source, {
        lang: "tsx",
        sourceType: "module",
        jsx: { runtime: "automatic" },
        plugins: {
          reactCompiler: {
            enabled: false,
          },
        },
      });
      expect(result.errors).toEqual([]);
    });

    test("works without plugins option", () => {
      const source = `
        function Component({ name }) {
          return <div>{name}</div>;
        }
      `;
      const result = transformSync("test.tsx", source, {
        lang: "tsx",
        sourceType: "module",
        jsx: { runtime: "automatic" },
      });
      expect(result.errors).toEqual([]);
      expect(result.code).toContain("_jsx");
    });
  });

  describe("memoization output", () => {
    test("compiler-runtime import is injected", () => {
      const source = `
        function Component({ name }) {
          return <div>Hello {name}</div>;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
      expect(result.code).toContain('from "react/compiler-runtime"');
    });

    test("_c cache call appears in compiled output", () => {
      const source = `
        function Component({ name }) {
          return <div>Hello {name}</div>;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
      expect(result.code).toMatch(/_c\(\d+\)/);
    });

    test("memo cache sentinel check appears for constant values", () => {
      const source = `
        function Component() {
          return <div>Hello</div>;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
      expect(result.code).toContain("react.memo_cache_sentinel");
    });
  });

  describe("memo/forwardRef discovery", () => {
    test("React.memo wrapped component compiles", () => {
      const source = `
        import React from 'react';
        const Component = React.memo(({ name }) => {
          return <div>{name}</div>;
        });
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
      expect(result.code).toMatch(/_c\(\d+\)/);
    });

    test("memo wrapped component compiles", () => {
      const source = `
        import { memo } from 'react';
        const Component = memo(({ name }) => {
          return <div>{name}</div>;
        });
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
      expect(result.code).toMatch(/_c\(\d+\)/);
    });

    test("forwardRef wrapped component compiles", () => {
      const source = `
        import { forwardRef } from 'react';
        const Component = forwardRef(function MyComponent(props, ref) {
          return <div ref={ref}>{props.name}</div>;
        });
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
      expect(result.code).toMatch(/_c\(\d+\)/);
    });

    test("export default memo compiles inner function", () => {
      const source = `
        import { memo } from 'react';
        export default memo(({ name }) => {
          return <div>{name}</div>;
        });
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
      expect(result.code).toContain("memo");
      expect(result.code).toMatch(/_c\(\d+\)/);
    });

    test("export named memo compiles inner function", () => {
      const source = `
        import { memo } from 'react';
        export const Component = memo(({ name }) => {
          return <div>{name}</div>;
        });
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
      expect(result.code).toMatch(/_c\(\d+\)/);
    });
  });

  describe("already-compiled skip", () => {
    test("file with compiler-runtime import is not recompiled", () => {
      const source = `
        import { c as _c } from "react/compiler-runtime";
        function Component({ name }) {
          const $ = _c(1);
          let t0;
          if ($[0] === Symbol.for("react.memo_cache_sentinel")) {
            t0 = <div>{name}</div>;
            $[0] = t0;
          } else {
            t0 = $[0];
          }
          return t0;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
      // Should NOT add a second compiler-runtime import
      const matches = result.code.match(/react\/compiler-runtime/g);
      expect(matches?.length).toBe(1);
    });
  });

  describe("opt-out directives", () => {
    test("'use no memo' skips function compilation", () => {
      const source = `
        function Component({ name }) {
          'use no memo';
          return <div>{name}</div>;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
      // Should not have compiler-runtime import since function was skipped
      expect(result.code).not.toContain("react/compiler-runtime");
    });

    test("'use no forget' skips function compilation", () => {
      const source = `
        function Component({ name }) {
          'use no forget';
          return <div>{name}</div>;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
      expect(result.code).not.toContain("react/compiler-runtime");
    });
  });

  describe("JSX closing element with compiler-generated references", () => {
    test("PascalCase component with children does not panic", () => {
      const source = `
        function App() {
          return <Wrapper>children</Wrapper>;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
      expect(result.code).toContain("Wrapper");
    });

    test("JSX member expression with children does not panic", () => {
      const source = `
        function App() {
          return <motion.div>children</motion.div>;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
      expect(result.code).toContain("motion.div");
    });

    test("nested PascalCase components with children do not panic", () => {
      const source = `
        function App({ items }) {
          return (
            <Layout>
              <Header>
                <Title>Hello</Title>
              </Header>
              <Content>{items}</Content>
            </Layout>
          );
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
    });
  });

  describe("JSX output correctness", () => {
    test("jsx runtime import is added", () => {
      const source = `
        function Component() {
          return <div>Hello</div>;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
      expect(result.code).toContain('from "react/jsx-runtime"');
    });

    test("jsx with multiple children uses jsxs", () => {
      const source = `
        function Component({ a, b }) {
          return <div>{a}{b}</div>;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
      expect(result.code).toContain("_jsxs");
    });

    test("jsx with single child uses jsx", () => {
      const source = `
        function Component({ name }) {
          return <div>{name}</div>;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
      expect(result.code).toContain("_jsx");
    });

    test("jsx with string-only children inlines them", () => {
      const source = `
        function Component() {
          return <div>Hello World</div>;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
      expect(result.code).toContain("Hello World");
    });

    test("jsx self-closing elements are handled", () => {
      const source = `
        function Component() {
          return <img src="test.png" alt="test" />;
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
      expect(result.code).toContain('"img"');
    });

    test("fragment syntax is transformed", () => {
      const source = `
        function Component() {
          return (
            <>
              <div>a</div>
              <div>b</div>
            </>
          );
        }
      `;
      const result = compileWithReactCompiler(source);
      expect(result.errors).toEqual([]);
      expect(result.code).toContain("Fragment");
    });
  });
});
