const y = 1;
expect(((function () {} as any) ?? y).name).toBe("");
