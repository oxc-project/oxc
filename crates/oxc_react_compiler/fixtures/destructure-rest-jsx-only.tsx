function Component({value, ...rest}) {
  return <Child {...rest} value={value} />;
}
