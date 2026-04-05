export const preprocess = () => "preprocessed";

const svelteConfig = {
  compilerOptions: {
    runes: true,
    generate: "dom",
  },
  preprocess,
};

export default svelteConfig;
