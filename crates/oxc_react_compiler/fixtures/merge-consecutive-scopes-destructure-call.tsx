declare function createContext(options: {
  settings: unknown;
  theme: unknown;
}): {Provider: React.ComponentType<{children: React.ReactNode}>};

function Component({value, settings, ...rest}) {
  const {Provider} = createContext({
    settings,
    theme: rest.theme,
  });
  return (
    <Outer {...rest}>
      <Provider>
        <Child value={value} />
      </Provider>
    </Outer>
  );
}
