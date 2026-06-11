import { greeting, server_settings } from "./config";

// Clean property access on annotated object consts.
const ok_host: string = server_settings.host;
const ok_total: number = server_settings.port + server_settings.retries;
const ok_concat: string = greeting.text + "!";
const ok_loud: boolean = greeting.loud;

// Missing properties (TS2339).
const bad_protocol = server_settings.protocol;
const bad_volume = greeting.volume;

// Arithmetic on string-typed properties.
const bad_double = server_settings.host * 2;
const bad_minus = greeting.text - 1;
const bad_rhs = 10 / server_settings.host;

// Arithmetic on number-typed properties stays clean.
const ok_doubled_port: number = server_settings.port * 2;
export const exported_summary: string = ok_host + ok_concat;
export const exported_count: number = ok_total + ok_doubled_port;
export const exported_loud: boolean = ok_loud;
