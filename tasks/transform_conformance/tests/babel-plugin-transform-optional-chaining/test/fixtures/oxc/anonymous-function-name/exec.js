expect((() => {})?.name).toBe("");
expect((function () {})?.name).toBe("");
expect((class {})?.name).toBe("");
