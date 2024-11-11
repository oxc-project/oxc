const Obj = {
  value: 0,
  async method() {
    super.value = true;
    () => {
      super['value'] = true;
      super.object.value = true;
    }
  }
}