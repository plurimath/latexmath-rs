#ifndef LATEXMATH_MODEL_ABI_H
#define LATEXMATH_MODEL_ABI_H
#include <stddef.h>
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
uint32_t latexmath_abi_version(void);
/* Buffers remain borrowed until return. JSON is UTF-8 and NUL-terminated.
   The result has {abi_version:1,ok:true,model:{...}} or {ok:false,error:{...}}.
   Free every result exactly once with latexmath_free_json_v1. */
char *latexmath_parse_model_json_v1(const uint8_t *input, size_t input_len,
                              const uint8_t *decimal, size_t decimal_len);
void latexmath_free_json_v1(char *result);
#ifdef __cplusplus
}
#endif
#endif
