function dec() {}
class Entity {
  #_name_accessor_storage = "";
  get name() {
    return this.#_name_accessor_storage;
  }
  set name(value) {
    this.#_name_accessor_storage = value;
  }
  #_count_accessor_storage = 0;
  get count() {
    return this.#_count_accessor_storage;
  }
  set count(value) {
    this.#_count_accessor_storage = value;
  }
  #_flag_accessor_storage = false;
  get flag() {
    return this.#_flag_accessor_storage;
  }
  set flag(value) {
    this.#_flag_accessor_storage = value;
  }
  #_untyped_accessor_storage = "x";
  get untyped() {
    return this.#_untyped_accessor_storage;
  }
  set untyped(value) {
    this.#_untyped_accessor_storage = value;
  }
  #_list_accessor_storage = [];
  get list() {
    return this.#_list_accessor_storage;
  }
  set list(value) {
    this.#_list_accessor_storage = value;
  }
  static #_sName_accessor_storage = "";
  static get sName() {
    return Entity.#_sName_accessor_storage;
  }
  static set sName(value) {
    Entity.#_sName_accessor_storage = value;
  }
  #_computed_computed_accessor_storage = 0;
  get ["computed"]() {
    return this.#_computed_computed_accessor_storage;
  }
  set ["computed"](value) {
    this.#_computed_computed_accessor_storage = value;
  }
}
babelHelpers.decorate([
  dec,
  babelHelpers.decorateMetadata("design:type", String),
  babelHelpers.decorateMetadata("design:paramtypes", [])
], Entity.prototype, "name", null);
babelHelpers.decorate([
  dec,
  babelHelpers.decorateMetadata("design:type", Number),
  babelHelpers.decorateMetadata("design:paramtypes", [])
], Entity.prototype, "count", null);
babelHelpers.decorate([
  dec,
  babelHelpers.decorateMetadata("design:type", Boolean),
  babelHelpers.decorateMetadata("design:paramtypes", [])
], Entity.prototype, "flag", null);
babelHelpers.decorate([
  dec,
  babelHelpers.decorateMetadata("design:type", Object),
  babelHelpers.decorateMetadata("design:paramtypes", [])
], Entity.prototype, "untyped", null);
babelHelpers.decorate([
  dec,
  babelHelpers.decorateMetadata("design:type", Array),
  babelHelpers.decorateMetadata("design:paramtypes", [])
], Entity.prototype, "list", null);
babelHelpers.decorate([
  dec,
  babelHelpers.decorateMetadata("design:type", String),
  babelHelpers.decorateMetadata("design:paramtypes", [])
], Entity, "sName", null);
babelHelpers.decorate([
  dec,
  babelHelpers.decorateMetadata("design:type", Number),
  babelHelpers.decorateMetadata("design:paramtypes", [])
], Entity.prototype, "computed", null);
