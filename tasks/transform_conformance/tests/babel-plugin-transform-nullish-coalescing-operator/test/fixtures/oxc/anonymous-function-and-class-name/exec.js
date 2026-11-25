var y;
expect((function () {} ?? y).name).toBe("");
expect(((() => {}) ?? y).name).toBe("");
expect((class {} ?? y).name).toBe("");
expect((function named() {} ?? y).name).toBe("named");
