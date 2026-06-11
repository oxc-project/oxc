// all clean: type-only imports and a namespace import through the chain
import type { Config, TopConfig } from "./top";
import * as configNs from "./top";

const ok_conf: Config = configNs.ok_buildConfig("app");
const ok_top: TopConfig = configNs.ok_makeConfig("svc");

export const ok_levelSum: number =
  ok_conf.level + ok_top.level + configNs.ok_defaultLevel;

// qualified type use through the namespace import
export type QualifiedConfig = configNs.Config;

export const ok_named: QualifiedConfig = configNs.ok_buildConfig("named");
