export interface UserConfig {
  base?: string | false;
}

declare const configDefaults: Required<UserConfig>;

export type Base = UserConfig["base"];

export function resolveBaseUrl(
  base: UserConfig["base"] = configDefaults.base,
  isBuild: boolean,
): string {
  return "";
}

export function resolveAliasedBaseUrl(
  base: Base = configDefaults.base,
  isBuild: boolean,
): string {
  return "";
}
