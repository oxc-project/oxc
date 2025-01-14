class Cls {
  prop = async () => (this, super.prop)
  static prop = async () => (this, super.prop)

  nested = () => {
    async () => (this, super.prop);
  }
  static nested = () => {
    async () => (this, super.prop);
  }
}
