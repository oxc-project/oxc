export default {
  meta: {
    name: 'error-plugin',
  },
  rules: {
    error: {
      createOnce(_context) {
        throw new Error('Whoops!');
      },
    },
  },
};
