export default {
  meta: {
    name: 'error-plugin',
  },
  rules: {
    error: {
      createOnce(_context) {
        return {
          after() {
            throw new Error('Whoops!');
          },
        };
      },
    },
  },
};
