const component = {
  hasMoreData: true,
  called: false,
  async addData() {
    this.called = true;
  },
  async loadMoreData() {
    if (!this.hasMoreData) return;
    await this.addData();
  },
};

return component.loadMoreData().then(function () {
  expect(component.called).toBe(true);
});
