export class C extends B {
  @convert(val => {
    if (['rect', 'triangle'].includes(val)) {
      return val;
    }

    return 'rect';
  })
  @derive(val => {
    if (val === 'triangle') {
      return {
        rotate: 0,
      };
    }
    return {};
  })
  @field()
  accessor shapeType: 'rect' | 'triangle' = 'rect';
}
