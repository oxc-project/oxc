// Round 3: effect read vs capture — v2
// Pattern: variable from ternary → object literal → forEach lambda mutation
// Source: ParseBusinessProfile.js
function useProfile(node) {
  const childNode = node.maybeChild('key');
  const value = childNode ? childNode.contentString() : undefined;
  const profile = {
    key_value: value,
  };
  Object.keys(profile).forEach(key => {
    if (profile[key] == null) {
      delete profile[key];
    }
  });
  return profile;
}


const sproutInputs = [[{
  maybeChild: () => {
    "use no memo";

    return null;
  }
}], [{
  maybeChild: () => {
    "use no memo";

    return {
      contentString: () => {
        "use no memo";

        return "updated";
      }
    };
  }
}]];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return useProfile(args[0]);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
