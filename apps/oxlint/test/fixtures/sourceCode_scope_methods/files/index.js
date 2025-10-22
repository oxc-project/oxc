const topLevelConstant = 1,
  secondTopLevelConstant = 2;

function topLevelFunction(param) {
  const localConstant = topLevelConstant + param;
  return function innerFunction() {
    return localConstant + Math.PI;
  };
}

export const topLevelExport = topLevelFunction(2);
