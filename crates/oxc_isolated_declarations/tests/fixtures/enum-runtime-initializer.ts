export enum BadEnum {
  A = Math.random(),
  B = A + 1,
}

export const enum BadConstEnum {
  A = Math.random(),
  B = A + 1,
}

export const enum TemplateEnum {
  NoSubstitution = `constant`,
  WithSubstitution = `prefix${Math.random()}`,
}
