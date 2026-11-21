import type { Opts, Unused } from "./opts";
import DefaultType from "./default";
import * as Types from "./types";
import { factory } from "./factory";

interface Internal { value: Dependency }
type Dependency = Opts;
interface UnusedLocal { unused: true }

export declare const defaults: Opts;
export declare let state: Internal;
export declare var other: DefaultType, qualified: Types.Options;
export declare const create: typeof factory;
export declare const literal = "literal";
