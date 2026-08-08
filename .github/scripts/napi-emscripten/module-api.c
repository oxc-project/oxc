#include <stdint.h>

// napi-rs registers its exports through `napi_register_wasm_v1`, but unlike
// Node's NAPI_MODULE_INIT macro it does not emit this version getter. Emnapi
// requires it when creating the environment.
int32_t node_api_module_get_api_version_v1(void) { return 4; }
