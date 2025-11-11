# Missing TSType Variants in ESTree Converter

## Summary

After investigating the `convert_ts_type` function in `crates/oxc_linter/src/estree_converter.rs`, I found that most TSType variants are implemented, but **4 variants are missing**:

## Missing Variants

### 1. TSNamedTupleMember
- **Status**: ❌ Not handled as a top-level TSType
- **Note**: Currently handled in `convert_ts_tuple_element` (line 6780), but not as a direct TSType variant
- **Usage**: Named tuple members like `[first: string, second: number]`
- **ESTree type**: `"TSNamedTupleMember"`
- **oxc AST**: `TSType::TSNamedTupleMember(Box<'a, TSNamedTupleMember<'a>>)`
- **Builder method**: `self.builder.ts_type_named_tuple_member(span, label, element_type, optional)`

### 2. JSDocNullableType
- **Status**: ❌ Not handled
- **Usage**: JSDoc type annotations like `{?number}` (nullable)
- **ESTree type**: `"JSDocNullableType"`
- **oxc AST**: `TSType::JSDocNullableType(Box<'a, JSDocNullableType<'a>>)`
- **Builder method**: `self.builder.alloc_js_doc_nullable_type(span, type_annotation)`

### 3. JSDocNonNullableType
- **Status**: ❌ Not handled
- **Usage**: JSDoc type annotations like `{!number}` (non-nullable)
- **ESTree type**: `"JSDocNonNullableType"`
- **oxc AST**: `TSType::JSDocNonNullableType(Box<'a, JSDocNonNullableType<'a>>)`
- **Builder method**: `self.builder.alloc_js_doc_non_nullable_type(span, type_annotation)`

### 4. JSDocUnknownType
- **Status**: ❌ Not handled
- **Usage**: JSDoc type annotations like `{*}` (unknown/any)
- **ESTree type**: `"JSDocUnknownType"`
- **oxc AST**: `TSType::JSDocUnknownType(Box<'a, JSDocUnknownType>)`
- **Builder method**: `self.builder.alloc_js_doc_unknown_type(span)`

## Currently Implemented Variants (✅)

All other TSType variants are implemented:
- ✅ All keyword types (TSAnyKeyword, TSBigIntKeyword, etc.)
- ✅ TSArrayType
- ✅ TSConditionalType
- ✅ TSConstructorType
- ✅ TSFunctionType
- ✅ TSImportType
- ✅ TSIndexedAccessType
- ✅ TSInferType
- ✅ TSIntersectionType
- ✅ TSLiteralType
- ✅ TSMappedType
- ✅ TSTemplateLiteralType
- ✅ TSThisType
- ✅ TSTupleType
- ✅ TSTypeLiteral
- ✅ TSTypeOperatorType
- ✅ TSTypePredicate
- ✅ TSTypeQuery
- ✅ TSTypeReference
- ✅ TSUnionType
- ✅ TSParenthesizedType

## Impact Assessment

### High Priority
- **TSNamedTupleMember**: May be encountered in real-world TypeScript code with named tuple types

### Low Priority
- **JSDoc types**: These are primarily used in JSDoc comments, which are less common in modern TypeScript codebases that use TypeScript syntax directly. However, they may appear when converting codebases that rely heavily on JSDoc annotations.

## Implementation Notes

1. **TSNamedTupleMember**: This is already partially handled in tuple elements, but needs to be added as a top-level TSType variant in the `convert_ts_type` match statement.

2. **JSDoc types**: These are simpler - they typically just wrap another type (except JSDocUnknownType which has no type annotation).

3. **Location**: All implementations should be added to the `convert_ts_type` function around line 4797-5627 in `crates/oxc_linter/src/estree_converter.rs`.

## Next Steps

1. Add match arms for the 4 missing variants in `convert_ts_type`
2. Implement conversion logic for each variant
3. Add tests to verify the conversions work correctly
4. Update status document

