import {Stringify} from 'shared-runtime';

function foo() {
  let oneNaN = 1;
  oneNaN **= 0 / 0;
  let oneInfinity = 1;
  oneInfinity **= 1 / 0;
  let oneNegativeInfinity = 1;
  oneNegativeInfinity **= -1 / 0;
  let negativeOneNaN = -1;
  negativeOneNaN **= 0 / 0;
  let negativeOneInfinity = -1;
  negativeOneInfinity **= 1 / 0;
  let negativeOneNegativeInfinity = -1;
  negativeOneNegativeInfinity **= -1 / 0;
  let nanZero = 0 / 0;
  nanZero **= 0;
  let nanNegativeZero = 0 / 0;
  nanNegativeZero **= -0;
  let twoInfinity = 2;
  twoInfinity **= 1 / 0;
  let halfInfinity = 0.5;
  halfInfinity **= 1 / 0;
  let twoNegativeInfinity = 2;
  twoNegativeInfinity **= -1 / 0;
  let halfNegativeInfinity = 0.5;
  halfNegativeInfinity **= -1 / 0;
  let negativeZeroOdd = -0;
  negativeZeroOdd **= 3;
  let negativeZeroEven = -0;
  negativeZeroEven **= 2;
  let negativeZeroNegativeOdd = -0;
  negativeZeroNegativeOdd **= -3;
  let negativeZeroNegativeEven = -0;
  negativeZeroNegativeEven **= -2;
  let negativeInfinityOdd = -1 / 0;
  negativeInfinityOdd **= 3;
  let negativeInfinityNegativeOdd = -1 / 0;
  negativeInfinityNegativeOdd **= -3;
  let negativeFractional = -2;
  negativeFractional **= 0.5;
  let finite = 2;
  finite **= 10;

  return (
    <Stringify
      value={[
        oneNaN,
        oneInfinity,
        oneNegativeInfinity,
        negativeOneNaN,
        negativeOneInfinity,
        negativeOneNegativeInfinity,
        nanZero,
        nanNegativeZero,
        twoInfinity,
        halfInfinity,
        twoNegativeInfinity,
        halfNegativeInfinity,
        negativeZeroOdd,
        negativeZeroEven,
        negativeZeroNegativeOdd,
        negativeZeroNegativeEven,
        negativeInfinityOdd,
        negativeInfinityNegativeOdd,
        negativeFractional,
        finite,
      ]}
    />
  );
}

export const FIXTURE_ENTRYPOINT = {
  fn: foo,
  params: [],
  isComponent: false,
};
