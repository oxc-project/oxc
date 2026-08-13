// @compilationMode:"infer"
import { forwardRef } from 'react';

const Grid = forwardRef(({ label }, ref) => {
  const renderGridComponent = () => <div ref={ref}>{label}</div>;
  return renderGridComponent();
});

const List = forwardRef(function ({ items }, ref) {
  const renderListComponent = () => <ul ref={ref}>{items}</ul>;
  return renderListComponent();
});

export { Grid, List };
