export interface Config {
  name: string;
  level: number;
}

export const ok_defaultLevel: number = 3;

export function ok_makeConfig(name: string): Config {
  return { name, level: ok_defaultLevel };
}
