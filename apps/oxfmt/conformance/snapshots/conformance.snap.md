## js-in-vue

### Option 1: 423/425 (99.53%)

```json
{"printWidth":80}
```

| File | Note |
| :--- | :--- |
| [externals/vue-vben-admin/@core/ui-kit/shadcn-ui/src/components/render-content/render-content.vue](diffs/js-in-vue/externals__vue-vben-admin__@core__ui-kit__shadcn-ui__src__components__render-content__render-content.vue.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific |
| [externals/vue-vben-admin/effects/common-ui/src/components/api-component/api-component.vue](diffs/js-in-vue/externals__vue-vben-admin__effects__common-ui__src__components__api-component__api-component.vue.md) | `<T = any,>() => {}` comma removed in ts-in-vue as like plain `.ts`, intentional divergence: Prettier keeps in ts-in-xxx, but not in ts-in-md and also plain `.ts`. It is only required for `.tsx` and `.mts|cts` |

### Option 2: 424/425 (99.76%)

```json
{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true}
```

| File | Note |
| :--- | :--- |
| [externals/vue-vben-admin/effects/common-ui/src/components/api-component/api-component.vue](diffs/js-in-vue/externals__vue-vben-admin__effects__common-ui__src__components__api-component__api-component.vue.md) | `<T = any,>() => {}` comma removed in ts-in-vue as like plain `.ts`, intentional divergence: Prettier keeps in ts-in-xxx, but not in ts-in-md and also plain `.ts`. It is only required for `.tsx` and `.mts|cts` |

## gql-in-js

### Option 1: 11/13 (84.62%)

```json
{"printWidth":80}
```

| File | Note |
| :--- | :--- |
| [edge-cases/gql-in-js/template-expression-indent.js](diffs/gql-in-js/edge-cases__gql-in-js__template-expression-indent.js.md) | We match Prettier main (prettier/prettier#19725); 3.9.6 still preserves source indent non-idempotently |
| [externals/prettier/js/multiparser-graphql/graphql-tag.js](diffs/gql-in-js/externals__prettier__js__multiparser-graphql__graphql-tag.js.md) | Prettier moves `query Test { # c` own-line comment to next line, we keep |

### Option 2: 11/13 (84.62%)

```json
{"printWidth":100}
```

| File | Note |
| :--- | :--- |
| [edge-cases/gql-in-js/template-expression-indent.js](diffs/gql-in-js/edge-cases__gql-in-js__template-expression-indent.js.md) | We match Prettier main (prettier/prettier#19725); 3.9.6 still preserves source indent non-idempotently |
| [externals/prettier/js/multiparser-graphql/graphql-tag.js](diffs/gql-in-js/externals__prettier__js__multiparser-graphql__graphql-tag.js.md) | Prettier moves `query Test { # c` own-line comment to next line, we keep |

## css-in-js

### Option 1: 19/21 (90.48%)

```json
{"printWidth":80}
```

| File | Note |
| :--- | :--- |
| [edge-cases/css-in-js/template-expression-indent.js](diffs/css-in-js/edge-cases__css-in-js__template-expression-indent.js.md) | We match Prettier main (prettier/prettier#19725); 3.9.6 still preserves source indent non-idempotently |
| [externals/prettier/js/multiparser-css/styled-components.js](diffs/css-in-js/externals__prettier__js__multiparser-css__styled-components.js.md) | `Xxx.extend` not recognized as tag |

### Option 2: 19/21 (90.48%)

```json
{"printWidth":100}
```

| File | Note |
| :--- | :--- |
| [edge-cases/css-in-js/template-expression-indent.js](diffs/css-in-js/edge-cases__css-in-js__template-expression-indent.js.md) | We match Prettier main (prettier/prettier#19725); 3.9.6 still preserves source indent non-idempotently |
| [externals/prettier/js/multiparser-css/styled-components.js](diffs/css-in-js/externals__prettier__js__multiparser-css__styled-components.js.md) | `Xxx.extend` not recognized as tag |

## html-in-js

### Option 1: 168/194 (86.60%)

```json
{"printWidth":80}
```

| File | Note |
| :--- | :--- |
| [edge-cases/html-in-js/template-expression-indent.js](diffs/html-in-js/edge-cases__html-in-js__template-expression-indent.js.md) | We match Prettier main (prettier/prettier#19725); 3.9.6 still preserves source indent non-idempotently |
| [externals/webawesome/badge/badge.ts](diffs/html-in-js/externals__webawesome__badge__badge.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific |
| [externals/webawesome/button/button.ts](diffs/html-in-js/externals__webawesome__button__button.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific |
| [externals/webawesome/callout/callout.ts](diffs/html-in-js/externals__webawesome__callout__callout.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific |
| [externals/webawesome/carousel/carousel.ts](diffs/html-in-js/externals__webawesome__carousel__carousel.ts.md) | We match Prettier main (prettier/prettier#19725); 3.9.6 still preserves source indent non-idempotently |
| [externals/webawesome/checkbox/checkbox.ts](diffs/html-in-js/externals__webawesome__checkbox__checkbox.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific |
| [externals/webawesome/color-picker/color-picker.ts](diffs/html-in-js/externals__webawesome__color-picker__color-picker.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific<br>We match Prettier main (prettier/prettier#19725); 3.9.6 still preserves source indent non-idempotently |
| [externals/webawesome/copy-button/copy-button.ts](diffs/html-in-js/externals__webawesome__copy-button__copy-button.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific |
| [externals/webawesome/details/details.ts](diffs/html-in-js/externals__webawesome__details__details.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific |
| [externals/webawesome/dropdown-item/dropdown-item.ts](diffs/html-in-js/externals__webawesome__dropdown-item__dropdown-item.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific |
| [externals/webawesome/dropdown/dropdown.ts](diffs/html-in-js/externals__webawesome__dropdown__dropdown.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific |
| [externals/webawesome/format-number/format-number.ts](diffs/html-in-js/externals__webawesome__format-number__format-number.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific |
| [externals/webawesome/input/input.ts](diffs/html-in-js/externals__webawesome__input__input.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific<br>We match Prettier main (prettier/prettier#19725); 3.9.6 still preserves source indent non-idempotently |
| [externals/webawesome/number-input/number-input.styles.ts](diffs/html-in-js/externals__webawesome__number-input__number-input.styles.ts.md) | Layout-only: Prettier's fill fit-check breaks inside `var()` args in a long `calc()`; ours breaks after the operator. See crates/oxc_formatter_css/AGENTS.md |
| [externals/webawesome/number-input/number-input.ts](diffs/html-in-js/externals__webawesome__number-input__number-input.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific |
| [externals/webawesome/page/page.styles.ts](diffs/html-in-js/externals__webawesome__page__page.styles.ts.md) | Layout-only: Prettier's fill fit-check breaks inside `::slotted()` after a long `:not(...)`; ours breaks inside `:not(...)`. See crates/oxc_formatter_css/AGENTS.md |
| [externals/webawesome/popup/popup.ts](diffs/html-in-js/externals__webawesome__popup__popup.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific |
| [externals/webawesome/qr-code/qr-code.ts](diffs/html-in-js/externals__webawesome__qr-code__qr-code.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific |
| [externals/webawesome/radio-group/radio-group.ts](diffs/html-in-js/externals__webawesome__radio-group__radio-group.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific |
| [externals/webawesome/radio/radio.ts](diffs/html-in-js/externals__webawesome__radio__radio.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific |
| [externals/webawesome/rating/rating.ts](diffs/html-in-js/externals__webawesome__rating__rating.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific |
| [externals/webawesome/select/select.ts](diffs/html-in-js/externals__webawesome__select__select.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific |
| [externals/webawesome/slider/slider.ts](diffs/html-in-js/externals__webawesome__slider__slider.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific |
| [externals/webawesome/switch/switch.ts](diffs/html-in-js/externals__webawesome__switch__switch.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific |
| [externals/webawesome/tag/tag.ts](diffs/html-in-js/externals__webawesome__tag__tag.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific |
| [externals/webawesome/textarea/textarea.ts](diffs/html-in-js/externals__webawesome__textarea__textarea.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific |

### Option 2: 181/194 (93.30%)

```json
{"printWidth":100,"htmlWhitespaceSensitivity":"ignore"}
```

| File | Note |
| :--- | :--- |
| [edge-cases/html-in-js/template-expression-indent.js](diffs/html-in-js/edge-cases__html-in-js__template-expression-indent.js.md) | We match Prettier main (prettier/prettier#19725); 3.9.6 still preserves source indent non-idempotently |
| [externals/webawesome/button/button.ts](diffs/html-in-js/externals__webawesome__button__button.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific |
| [externals/webawesome/callout/callout.ts](diffs/html-in-js/externals__webawesome__callout__callout.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific |
| [externals/webawesome/carousel/carousel.ts](diffs/html-in-js/externals__webawesome__carousel__carousel.ts.md) | We match Prettier main (prettier/prettier#19725); 3.9.6 still preserves source indent non-idempotently |
| [externals/webawesome/color-picker/color-picker.ts](diffs/html-in-js/externals__webawesome__color-picker__color-picker.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific<br>We match Prettier main (prettier/prettier#19725); 3.9.6 still preserves source indent non-idempotently |
| [externals/webawesome/copy-button/copy-button.ts](diffs/html-in-js/externals__webawesome__copy-button__copy-button.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific |
| [externals/webawesome/format-number/format-number.ts](diffs/html-in-js/externals__webawesome__format-number__format-number.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific |
| [externals/webawesome/icon/icon.ts](diffs/html-in-js/externals__webawesome__icon__icon.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific |
| [externals/webawesome/input/input.ts](diffs/html-in-js/externals__webawesome__input__input.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific<br>We match Prettier main (prettier/prettier#19725); 3.9.6 still preserves source indent non-idempotently |
| [externals/webawesome/page/page.ts](diffs/html-in-js/externals__webawesome__page__page.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific |
| [externals/webawesome/popup/popup.ts](diffs/html-in-js/externals__webawesome__popup__popup.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific |
| [externals/webawesome/slider/slider.ts](diffs/html-in-js/externals__webawesome__slider__slider.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific |
| [externals/webawesome/textarea/textarea.ts](diffs/html-in-js/externals__webawesome__textarea__textarea.ts.md) | Union broken out of its `:`/`as` position: Prettier retries the whole union flat on the indented next line, we expand to leading-`|` members right away. Core oxc_formatter (plain `.ts` too), not embed-specific |

## angular-in-js

### Option 1: 7/7 (100.00%)

```json
{"printWidth":80}
```

### Option 2: 7/7 (100.00%)

```json
{"printWidth":100,"htmlWhitespaceSensitivity":"ignore"}
```

## md-in-js

### Option 1: 8/8 (100.00%)

```json
{"printWidth":80}
```

### Option 2: 8/8 (100.00%)

```json
{"printWidth":100,"proseWrap":"always"}
```

## xxx-in-js-comment

### Option 1: 4/5 (80.00%)

```json
{"printWidth":80}
```

| File | Note |
| :--- | :--- |
| [externals/prettier/js/multiparser-comments/comment-inside.js](diffs/xxx-in-js-comment/externals__prettier__js__multiparser-comments__comment-inside.js.md) | Broken `${}` holding comments: Prettier prints the expression at root indent (drops the embed indent), we indent to the placeholder |

### Option 2: 4/5 (80.00%)

```json
{"printWidth":100}
```

| File | Note |
| :--- | :--- |
| [externals/prettier/js/multiparser-comments/comment-inside.js](diffs/xxx-in-js-comment/externals__prettier__js__multiparser-comments__comment-inside.js.md) | Broken `${}` holding comments: Prettier prints the expression at root indent (drops the embed indent), we indent to the placeholder |

## svelte

### Option 1: 79/79 (100.00%)

```json
{"printWidth":80,"svelte":{}}
```

### Option 2: 79/79 (100.00%)

```json
{"printWidth":120,"singleQuote":true,"htmlWhitespaceSensitivity":"ignore","bracketSameLine":true,"svelteIndentScriptAndStyle":true,"svelteSortOrder":"options-scripts-styles-markup","svelte":{"indentScriptAndStyle":true,"sortOrder":"options-scripts-styles-markup"}}
```

## graphql

### Option 1: 712/712 (100.00%)

```json
{"printWidth":80}
```

### Option 2: 712/712 (100.00%)

```json
{"printWidth":100}
```

## less

### Option 1: 403/409 (98.53%)

```json
{"printWidth":80}
```

| File | Note |
| :--- | :--- |
| [externals/ng-zorro-antd/components/style/themes/compact.less](diffs/less/externals__ng-zorro-antd__components__style__themes__compact.less.md) | Allowed (layout-only): nested Less math — Prettier's fill fit-check breaks inside the wide chunk, ours breaks the separator (biome fill). See crates/oxc_formatter_css/AGENTS.md |
| [externals/ng-zorro-antd/components/style/themes/dark.less](diffs/less/externals__ng-zorro-antd__components__style__themes__dark.less.md) | Allowed: trailing `// comment` rides a line_suffix, never counts toward print width; Prettier only treats CSS-family `//` inline and breaks the value. See crates/oxc_formatter_css/AGENTS.md |
| [externals/ng-zorro-antd/components/style/themes/default.less](diffs/less/externals__ng-zorro-antd__components__style__themes__default.less.md) | Allowed (layout-only): nested Less math — Prettier's fill fit-check breaks inside the wide chunk, ours breaks the separator (biome fill). See crates/oxc_formatter_css/AGENTS.md<br>Allowed: trailing `// comment` rides a line_suffix, never counts toward print width; Prettier only treats CSS-family `//` inline and breaks the value. See crates/oxc_formatter_css/AGENTS.md |
| [externals/ng-zorro-antd/components/style/themes/variable.less](diffs/less/externals__ng-zorro-antd__components__style__themes__variable.less.md) | Allowed (layout-only): nested Less math — Prettier's fill fit-check breaks inside the wide chunk, ours breaks the separator (biome fill). See crates/oxc_formatter_css/AGENTS.md<br>Allowed: trailing `// comment` rides a line_suffix, never counts toward print width; Prettier only treats CSS-family `//` inline and breaks the value. See crates/oxc_formatter_css/AGENTS.md |
| [externals/ng-zorro-antd/components/table/style/index.less](diffs/less/externals__ng-zorro-antd__components__table__style__index.less.md) | Allowed (layout-only): nested Less math — Prettier's fill fit-check breaks inside the wide chunk, ours breaks the separator (biome fill). See crates/oxc_formatter_css/AGENTS.md |
| [externals/ng-zorro-antd/components/table/style/rtl.less](diffs/less/externals__ng-zorro-antd__components__table__style__rtl.less.md) | Allowed (layout-only): nested Less math — Prettier's fill fit-check breaks inside the wide chunk, ours breaks the separator (biome fill). See crates/oxc_formatter_css/AGENTS.md |

### Option 2: 406/409 (99.27%)

```json
{"printWidth":100}
```

| File | Note |
| :--- | :--- |
| [externals/ng-zorro-antd/components/style/themes/default.less](diffs/less/externals__ng-zorro-antd__components__style__themes__default.less.md) | Allowed (layout-only): nested Less math — Prettier's fill fit-check breaks inside the wide chunk, ours breaks the separator (biome fill). See crates/oxc_formatter_css/AGENTS.md<br>Allowed: trailing `// comment` rides a line_suffix, never counts toward print width; Prettier only treats CSS-family `//` inline and breaks the value. See crates/oxc_formatter_css/AGENTS.md |
| [externals/ng-zorro-antd/components/style/themes/variable.less](diffs/less/externals__ng-zorro-antd__components__style__themes__variable.less.md) | Allowed (layout-only): nested Less math — Prettier's fill fit-check breaks inside the wide chunk, ours breaks the separator (biome fill). See crates/oxc_formatter_css/AGENTS.md<br>Allowed: trailing `// comment` rides a line_suffix, never counts toward print width; Prettier only treats CSS-family `//` inline and breaks the value. See crates/oxc_formatter_css/AGENTS.md |
| [externals/ng-zorro-antd/components/table/style/rtl.less](diffs/less/externals__ng-zorro-antd__components__table__style__rtl.less.md) | Allowed (layout-only): nested Less math — Prettier's fill fit-check breaks inside the wide chunk, ours breaks the separator (biome fill). See crates/oxc_formatter_css/AGENTS.md |

## css

### Option 1: 221/221 (100.00%)

```json
{"printWidth":80}
```

### Option 2: 221/221 (100.00%)

```json
{"printWidth":100}
```

## yaml

### Option 1: 301/302 (99.67%)

```json
{"printWidth":80}
```

| File | Note |
| :--- | :--- |
| [externals/aws-cloudformation-templates/RainModules/load-balancer.yml](diffs/yaml/externals__aws-cloudformation-templates__RainModules__load-balancer.yml.md) | Allowed: over-indented comment after `key: value` (Prettier breaks the pair onto two lines because of comment indentation). See crates/oxc_formatter_yaml/AGENTS.md |

### Option 2: 301/302 (99.67%)

```json
{"printWidth":100,"tabWidth":4,"proseWrap":"always"}
```

| File | Note |
| :--- | :--- |
| [externals/aws-cloudformation-templates/RainModules/load-balancer.yml](diffs/yaml/externals__aws-cloudformation-templates__RainModules__load-balancer.yml.md) | Allowed: over-indented comment after `key: value` (Prettier breaks the pair onto two lines because of comment indentation). See crates/oxc_formatter_yaml/AGENTS.md |

### Option 3: 301/302 (99.67%)

```json
{"printWidth":120,"singleQuote":true,"bracketSpacing":false,"trailingComma":"none"}
```

| File | Note |
| :--- | :--- |
| [externals/aws-cloudformation-templates/RainModules/load-balancer.yml](diffs/yaml/externals__aws-cloudformation-templates__RainModules__load-balancer.yml.md) | Allowed: over-indented comment after `key: value` (Prettier breaks the pair onto two lines because of comment indentation). See crates/oxc_formatter_yaml/AGENTS.md |

## scss

### Option 1: 203/217 (93.55%)

```json
{"printWidth":80}
```

| File | Note |
| :--- | :--- |
| [externals/gitlab/stylesheets/components/content_editor.scss](diffs/scss/externals__gitlab__stylesheets__components__content_editor.scss.md) | Allowed (layout-only): `box-shadow` with `#{}` math — Prettier's fill fit-check breaks inside the wide chunk, ours breaks the separator (biome fill). See crates/oxc_formatter_css/AGENTS.md |
| [externals/gitlab/stylesheets/framework/diffs.scss](diffs/scss/externals__gitlab__stylesheets__framework__diffs.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/framework/variables_overrides.scss](diffs/scss/externals__gitlab__stylesheets__framework__variables_overrides.scss.md) | Allowed (semantics): Prettier adds a trailing comma to non-comma-list map-item parens (`1: ($spacer * 0.5)` → 1-element list); we keep them inline. See crates/oxc_formatter_css/AGENTS.md |
| [externals/gitlab/stylesheets/highlight/conflict_colors.scss](diffs/scss/externals__gitlab__stylesheets__highlight__conflict_colors.scss.md) | Allowed: Prettier drops blank lines in SCSS maps with paren values; ours preserves (prettier/prettier#16824) |
| [externals/gitlab/stylesheets/page_bundles/_ide_theme_overrides.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles___ide_theme_overrides.scss.md) | Layout-only: Prettier's fill fit-check breaks inside `var()` args in a long `calc()`; ours breaks after the operator. See crates/oxc_formatter_css/AGENTS.md |
| [externals/gitlab/stylesheets/page_bundles/editor.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__editor.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/page_bundles/environments.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__environments.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/page_bundles/issuable_list.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__issuable_list.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/page_bundles/labels.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__labels.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/page_bundles/merge_requests.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__merge_requests.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/page_bundles/projects.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__projects.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/page_bundles/settings.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__settings.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/pages/profile.scss](diffs/scss/externals__gitlab__stylesheets__pages__profile.scss.md) | Allowed: trailing `// comment` rides a line_suffix, never counts toward print width; Prettier only treats CSS-family `//` inline and breaks the value. See crates/oxc_formatter_css/AGENTS.md |
| [externals/gitlab/stylesheets/pages/settings.scss](diffs/scss/externals__gitlab__stylesheets__pages__settings.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |

### Option 2: 204/217 (94.01%)

```json
{"printWidth":100}
```

| File | Note |
| :--- | :--- |
| [externals/gitlab/stylesheets/framework/diffs.scss](diffs/scss/externals__gitlab__stylesheets__framework__diffs.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/framework/sidebar.scss](diffs/scss/externals__gitlab__stylesheets__framework__sidebar.scss.md) | long-expr line-break position |
| [externals/gitlab/stylesheets/framework/variables_overrides.scss](diffs/scss/externals__gitlab__stylesheets__framework__variables_overrides.scss.md) | Allowed (semantics): Prettier adds a trailing comma to non-comma-list map-item parens (`1: ($spacer * 0.5)` → 1-element list); we keep them inline. See crates/oxc_formatter_css/AGENTS.md |
| [externals/gitlab/stylesheets/highlight/conflict_colors.scss](diffs/scss/externals__gitlab__stylesheets__highlight__conflict_colors.scss.md) | Allowed: Prettier drops blank lines in SCSS maps with paren values; ours preserves (prettier/prettier#16824) |
| [externals/gitlab/stylesheets/page_bundles/_ide_theme_overrides.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles___ide_theme_overrides.scss.md) | Layout-only: Prettier's fill fit-check breaks inside `var()` args in a long `calc()`; ours breaks after the operator. See crates/oxc_formatter_css/AGENTS.md |
| [externals/gitlab/stylesheets/page_bundles/editor.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__editor.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/page_bundles/environments.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__environments.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/page_bundles/issuable_list.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__issuable_list.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/page_bundles/labels.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__labels.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/page_bundles/merge_requests.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__merge_requests.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/page_bundles/projects.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__projects.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/page_bundles/settings.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__settings.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/pages/settings.scss](diffs/scss/externals__gitlab__stylesheets__pages__settings.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
