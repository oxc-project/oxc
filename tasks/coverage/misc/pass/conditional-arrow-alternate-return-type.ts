const formErrors = { foo: 'bar' };

type Form = { id: number; errors: Record<string, boolean> };

type Errors<T extends Form> = Pick<T, 'errors'>;

type FooBarForm = Form & { tag: 'foobarform' };

const getFormError =
  typeof formErrors !== 'object'
    ? () => ({})
    : (index: number): Errors<FooBarForm> => {
        return {
          errors: { a: true, b: false },
        };
      };
