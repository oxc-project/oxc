<script lang="ts" setup>
// Deeply nested call expressions whose arguments are themselves nested call expressions
// produced a `BestFitting` IR with each variant duplicating the inner content.
// Cloning the converted JSON across variants blew up the Prettier Doc size exponentially
// (~48 MB at 3 levels), and effectively hung.
//
// `oxc_formatter::Interned` sub-trees are now emitted once into a `_oxfmtRefs`
// array and referenced via `_oxfmtRef` placeholders, preserving sharing across the JSON boundary.
// https://github.com/oxc-project/oxc/issues/22350
const schema = computed(() =>
  yup.object({
    a: yup.object({
      b: yup.object({
        c: yup.object().when(() =>
          yup.object({
            d: yup.object().when(() =>
              yup.object({
                e: yup.object({ f: yup.object({ g: yup.string() }) }),
              }),
            ),
          }),
        ),
      }),
    }),
  }),
);
</script>
