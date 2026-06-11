// Clean-only fixture: every assignment here is structurally valid.
interface Config {
  host: string;
  port: number;
  timeout_ms?: number;
}

interface TlsConfig extends Config {
  cert_path: string;
}

// Extra member allowed through an intermediate variable (width subtyping).
const base_value = { host: "localhost", port: 8080, retries: 3 };
const ok_widened: Config = base_value;

const ok_optional_omitted: Config = { host: "h", port: 80 };
const ok_optional_present: Config = { host: "h", port: 80, timeout_ms: 500 };

const ok_tls: TlsConfig = { host: "secure", port: 443, cert_path: "/tmp/cert" };
const ok_tls_as_base: Config = ok_tls;

function readPort(config: Config): number {
  return config.port;
}
const ok_port_from_derived: number = readPort(ok_tls);

// Nested type-literal target, fully satisfied.
type Wrapper = { payload: { id: number; note?: string } };
const ok_wrapper: Wrapper = { payload: { id: 7 } };

export {};
