// @compilationMode:"infer" @panicThreshold:"none"
export function Component({tag, consume}) {
  return consume(tag`value`);
}


const sproutInputs = [[{
  tag: strings => {
    "use no memo";

    return strings.join(":");
  },
  consume: value => {
    "use no memo";

    return value + "!";
  }
}], [{
  tag: strings => {
    "use no memo";

    return strings.join(":");
  },
  consume: value => {
    "use no memo";

    return value + "?";
  }
}]];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
