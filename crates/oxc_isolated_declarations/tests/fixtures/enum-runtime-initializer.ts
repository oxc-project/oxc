export enum BadEnum {
  A = Math.random(),
  B = A + 1,
}

export const enum BadConstEnum {
  A = Math.random(),
  B = A + 1,
}

const foo = "123";

export const enum TemplateEnum {
  NoSubstitution = `constant`,
  WithConstantSubstitution = `prefix${"foo"}`,
  WithConstantIdentifierSubstitution = `prefix${foo}`,
  WithRuntimeSubstitution = `prefix${Math.random()}`,
}
