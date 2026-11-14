export let Status = /* @__PURE__ */ function(Status) {
  Status[Status["A"] = 1] = "A";
  return Status;
}({});

export let Name = /* @__PURE__ */ function(Name) {
  Name["A"] = "a";
  return Name;
}({});

class U {
  status;
  name;
}

babelHelpers.decorate([Dec(), babelHelpers.decorateMetadata("design:type", Number)], U.prototype, "status", void 0);
babelHelpers.decorate([Dec(), babelHelpers.decorateMetadata("design:type", String)], U.prototype, "name", void 0);
