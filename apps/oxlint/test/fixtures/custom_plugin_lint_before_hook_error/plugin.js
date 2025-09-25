export default {
  meta: {
    name: 'error-plugin',
  },
  rules: {
    error: {
      createOnce(_context) {
        return {
          before() {
            throw new Error('Whoops!');
          },
        };
      },
    },
  },
};
