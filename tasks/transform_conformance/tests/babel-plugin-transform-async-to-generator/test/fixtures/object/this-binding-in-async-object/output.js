const component = { methods: { loadMoreData() {
  var _this = this;
  return babelHelpers.asyncToGenerator(function* () {
    if (!_this.hasMoreData) return;
    yield _this.addData();
  })();
} } };
export default component;
