function Component(props) {
  let x;
  if (props.cond) {
    switch (props.test) {
      case 0: {
        x = props.v0;
        break;
      }
      case 1: {
        x = props.v1;
        break;
      }
      case 2: {
      }
      default: {
        x = props.v2;
      }
    }
  } else {
    if (props.cond2) {
      x = props.b;
    } else {
      x = props.c;
    }
  }
  x;
}

const sproutInputs = [
  [{cond:true,test:0,v0:1,v1:2,v2:3}],
  [{cond:true,test:2,v0:4,v1:5,v2:6}],
  [{cond:false,cond2:true,b:7,c:8}],
  [{cond:false,cond2:false,b:9,c:10}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[2], sproutInputs[2], sproutInputs[3], sproutInputs[3], sproutInputs[0]],
};
