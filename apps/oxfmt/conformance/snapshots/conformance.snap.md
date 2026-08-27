## js-in-vue

### Option 1: 423/426 (99.30%)

```json
{"printWidth":80}
```

| File | Note |
| :--- | :--- |
| [edge-cases/js-in-vue/generic-trailing-comma.vue](diffs/js-in-vue/edge-cases__js-in-vue__generic-trailing-comma.vue.md) | `<T = any,>` comma removed like plain `.ts`. See apps/oxfmt/DIVERGENCES.md#ts-in-vue-generic-trailing-comma |
| [externals/vue-vben-admin/@core/ui-kit/shadcn-ui/src/components/render-content/render-content.vue](diffs/js-in-vue/externals__vue-vben-admin__@core__ui-kit__shadcn-ui__src__components__render-content__render-content.vue.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry |
| [externals/vue-vben-admin/effects/common-ui/src/components/api-component/api-component.vue](diffs/js-in-vue/externals__vue-vben-admin__effects__common-ui__src__components__api-component__api-component.vue.md) | `<T = any,>` comma removed like plain `.ts`. See apps/oxfmt/DIVERGENCES.md#ts-in-vue-generic-trailing-comma<br>union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry |

### Option 2: 424/426 (99.53%)

```json
{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true}
```

| File | Note |
| :--- | :--- |
| [edge-cases/js-in-vue/generic-trailing-comma.vue](diffs/js-in-vue/edge-cases__js-in-vue__generic-trailing-comma.vue.md) | `<T = any,>` comma removed like plain `.ts`. See apps/oxfmt/DIVERGENCES.md#ts-in-vue-generic-trailing-comma |
| [externals/vue-vben-admin/effects/common-ui/src/components/api-component/api-component.vue](diffs/js-in-vue/externals__vue-vben-admin__effects__common-ui__src__components__api-component__api-component.vue.md) | `<T = any,>` comma removed like plain `.ts`. See apps/oxfmt/DIVERGENCES.md#ts-in-vue-generic-trailing-comma<br>union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry |

## gql-in-js

### Option 1: 10/12 (83.33%)

```json
{"printWidth":80}
```

| File | Note |
| :--- | :--- |
| [edge-cases/gql-in-js/template-expression-indent.js](diffs/gql-in-js/edge-cases__gql-in-js__template-expression-indent.js.md) | embedded `${expr}` re-indents to the placeholder. See apps/oxfmt/DIVERGENCES.md#template-expression-indent |
| [externals/prettier/js/multiparser-graphql/graphql-tag.js](diffs/gql-in-js/externals__prettier__js__multiparser-graphql__graphql-tag.js.md) | `{ # c` comment after an opening delimiter stays inline. See crates/oxc_formatter_graphql/DIVERGENCES.md#comment-after-opening-delimiter |

### Option 2: 10/12 (83.33%)

```json
{"printWidth":100}
```

| File | Note |
| :--- | :--- |
| [edge-cases/gql-in-js/template-expression-indent.js](diffs/gql-in-js/edge-cases__gql-in-js__template-expression-indent.js.md) | embedded `${expr}` re-indents to the placeholder. See apps/oxfmt/DIVERGENCES.md#template-expression-indent |
| [externals/prettier/js/multiparser-graphql/graphql-tag.js](diffs/gql-in-js/externals__prettier__js__multiparser-graphql__graphql-tag.js.md) | `{ # c` comment after an opening delimiter stays inline. See crates/oxc_formatter_graphql/DIVERGENCES.md#comment-after-opening-delimiter |

## css-in-js

### Option 1: 18/21 (85.71%)

```json
{"printWidth":80}
```

| File | Note |
| :--- | :--- |
| [edge-cases/css-in-js/styled-extend-tag.js](diffs/css-in-js/edge-cases__css-in-js__styled-extend-tag.js.md) | `Xxx.extend` not recognized as tag. See apps/oxfmt/DIVERGENCES.md#styled-extend-tag |
| [edge-cases/css-in-js/template-expression-indent.js](diffs/css-in-js/edge-cases__css-in-js__template-expression-indent.js.md) | embedded `${expr}` re-indents to the placeholder. See apps/oxfmt/DIVERGENCES.md#template-expression-indent |
| [externals/prettier/js/multiparser-css/styled-components.js](diffs/css-in-js/externals__prettier__js__multiparser-css__styled-components.js.md) | `Xxx.extend` not recognized as tag. See apps/oxfmt/DIVERGENCES.md#styled-extend-tag |

### Option 2: 18/21 (85.71%)

```json
{"printWidth":100}
```

| File | Note |
| :--- | :--- |
| [edge-cases/css-in-js/styled-extend-tag.js](diffs/css-in-js/edge-cases__css-in-js__styled-extend-tag.js.md) | `Xxx.extend` not recognized as tag. See apps/oxfmt/DIVERGENCES.md#styled-extend-tag |
| [edge-cases/css-in-js/template-expression-indent.js](diffs/css-in-js/edge-cases__css-in-js__template-expression-indent.js.md) | embedded `${expr}` re-indents to the placeholder. See apps/oxfmt/DIVERGENCES.md#template-expression-indent |
| [externals/prettier/js/multiparser-css/styled-components.js](diffs/css-in-js/externals__prettier__js__multiparser-css__styled-components.js.md) | `Xxx.extend` not recognized as tag. See apps/oxfmt/DIVERGENCES.md#styled-extend-tag |

## html-in-js

### Option 1: 168/194 (86.60%)

```json
{"printWidth":80}
```

| File | Note |
| :--- | :--- |
| [edge-cases/html-in-js/template-expression-indent.js](diffs/html-in-js/edge-cases__html-in-js__template-expression-indent.js.md) | embedded `${expr}` re-indents to the placeholder. See apps/oxfmt/DIVERGENCES.md#template-expression-indent |
| [externals/webawesome/badge/badge.ts](diffs/html-in-js/externals__webawesome__badge__badge.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry |
| [externals/webawesome/button/button.ts](diffs/html-in-js/externals__webawesome__button__button.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry |
| [externals/webawesome/callout/callout.ts](diffs/html-in-js/externals__webawesome__callout__callout.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry |
| [externals/webawesome/carousel/carousel.ts](diffs/html-in-js/externals__webawesome__carousel__carousel.ts.md) | embedded `${expr}` re-indents to the placeholder. See apps/oxfmt/DIVERGENCES.md#template-expression-indent |
| [externals/webawesome/checkbox/checkbox.ts](diffs/html-in-js/externals__webawesome__checkbox__checkbox.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry |
| [externals/webawesome/color-picker/color-picker.ts](diffs/html-in-js/externals__webawesome__color-picker__color-picker.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry<br>embedded `${expr}` re-indents to the placeholder. See apps/oxfmt/DIVERGENCES.md#template-expression-indent |
| [externals/webawesome/copy-button/copy-button.ts](diffs/html-in-js/externals__webawesome__copy-button__copy-button.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry |
| [externals/webawesome/details/details.ts](diffs/html-in-js/externals__webawesome__details__details.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry |
| [externals/webawesome/dropdown-item/dropdown-item.ts](diffs/html-in-js/externals__webawesome__dropdown-item__dropdown-item.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry |
| [externals/webawesome/dropdown/dropdown.ts](diffs/html-in-js/externals__webawesome__dropdown__dropdown.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry |
| [externals/webawesome/format-number/format-number.ts](diffs/html-in-js/externals__webawesome__format-number__format-number.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry |
| [externals/webawesome/input/input.ts](diffs/html-in-js/externals__webawesome__input__input.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry<br>embedded `${expr}` re-indents to the placeholder. See apps/oxfmt/DIVERGENCES.md#template-expression-indent |
| [externals/webawesome/number-input/number-input.styles.ts](diffs/html-in-js/externals__webawesome__number-input__number-input.styles.ts.md) | fill break position (Prettier breaks inside the wide chunk, ours at the separator). See crates/oxc_formatter_css/DIVERGENCES.md#fill-break-position |
| [externals/webawesome/number-input/number-input.ts](diffs/html-in-js/externals__webawesome__number-input__number-input.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry |
| [externals/webawesome/page/page.styles.ts](diffs/html-in-js/externals__webawesome__page__page.styles.ts.md) | fill break position (Prettier breaks inside the wide chunk, ours at the separator). See crates/oxc_formatter_css/DIVERGENCES.md#fill-break-position |
| [externals/webawesome/popup/popup.ts](diffs/html-in-js/externals__webawesome__popup__popup.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry |
| [externals/webawesome/qr-code/qr-code.ts](diffs/html-in-js/externals__webawesome__qr-code__qr-code.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry |
| [externals/webawesome/radio-group/radio-group.ts](diffs/html-in-js/externals__webawesome__radio-group__radio-group.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry |
| [externals/webawesome/radio/radio.ts](diffs/html-in-js/externals__webawesome__radio__radio.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry |
| [externals/webawesome/rating/rating.ts](diffs/html-in-js/externals__webawesome__rating__rating.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry |
| [externals/webawesome/select/select.ts](diffs/html-in-js/externals__webawesome__select__select.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry |
| [externals/webawesome/slider/slider.ts](diffs/html-in-js/externals__webawesome__slider__slider.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry |
| [externals/webawesome/switch/switch.ts](diffs/html-in-js/externals__webawesome__switch__switch.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry |
| [externals/webawesome/tag/tag.ts](diffs/html-in-js/externals__webawesome__tag__tag.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry |
| [externals/webawesome/textarea/textarea.ts](diffs/html-in-js/externals__webawesome__textarea__textarea.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry |

### Option 2: 181/194 (93.30%)

```json
{"printWidth":100,"htmlWhitespaceSensitivity":"ignore"}
```

| File | Note |
| :--- | :--- |
| [edge-cases/html-in-js/template-expression-indent.js](diffs/html-in-js/edge-cases__html-in-js__template-expression-indent.js.md) | embedded `${expr}` re-indents to the placeholder. See apps/oxfmt/DIVERGENCES.md#template-expression-indent |
| [externals/webawesome/button/button.ts](diffs/html-in-js/externals__webawesome__button__button.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry |
| [externals/webawesome/callout/callout.ts](diffs/html-in-js/externals__webawesome__callout__callout.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry |
| [externals/webawesome/carousel/carousel.ts](diffs/html-in-js/externals__webawesome__carousel__carousel.ts.md) | embedded `${expr}` re-indents to the placeholder. See apps/oxfmt/DIVERGENCES.md#template-expression-indent |
| [externals/webawesome/color-picker/color-picker.ts](diffs/html-in-js/externals__webawesome__color-picker__color-picker.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry<br>embedded `${expr}` re-indents to the placeholder. See apps/oxfmt/DIVERGENCES.md#template-expression-indent |
| [externals/webawesome/copy-button/copy-button.ts](diffs/html-in-js/externals__webawesome__copy-button__copy-button.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry |
| [externals/webawesome/format-number/format-number.ts](diffs/html-in-js/externals__webawesome__format-number__format-number.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry |
| [externals/webawesome/icon/icon.ts](diffs/html-in-js/externals__webawesome__icon__icon.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry |
| [externals/webawesome/input/input.ts](diffs/html-in-js/externals__webawesome__input__input.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry<br>embedded `${expr}` re-indents to the placeholder. See apps/oxfmt/DIVERGENCES.md#template-expression-indent |
| [externals/webawesome/page/page.ts](diffs/html-in-js/externals__webawesome__page__page.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry |
| [externals/webawesome/popup/popup.ts](diffs/html-in-js/externals__webawesome__popup__popup.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry |
| [externals/webawesome/slider/slider.ts](diffs/html-in-js/externals__webawesome__slider__slider.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry |
| [externals/webawesome/textarea/textarea.ts](diffs/html-in-js/externals__webawesome__textarea__textarea.ts.md) | union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry |

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

### Option 1: 4/6 (66.67%)

```json
{"printWidth":80}
```

| File | Note |
| :--- | :--- |
| [edge-cases/xxx-in-js-comment/broken-template-comment-indent.js](diffs/xxx-in-js-comment/edge-cases__xxx-in-js-comment__broken-template-comment-indent.js.md) | broken `${}` holding comments indents to the placeholder. See apps/oxfmt/DIVERGENCES.md#broken-template-comment-indent |
| [externals/prettier/js/multiparser-comments/comment-inside.js](diffs/xxx-in-js-comment/externals__prettier__js__multiparser-comments__comment-inside.js.md) | broken `${}` holding comments indents to the placeholder. See apps/oxfmt/DIVERGENCES.md#broken-template-comment-indent |

### Option 2: 4/6 (66.67%)

```json
{"printWidth":100}
```

| File | Note |
| :--- | :--- |
| [edge-cases/xxx-in-js-comment/broken-template-comment-indent.js](diffs/xxx-in-js-comment/edge-cases__xxx-in-js-comment__broken-template-comment-indent.js.md) | broken `${}` holding comments indents to the placeholder. See apps/oxfmt/DIVERGENCES.md#broken-template-comment-indent |
| [externals/prettier/js/multiparser-comments/comment-inside.js](diffs/xxx-in-js-comment/externals__prettier__js__multiparser-comments__comment-inside.js.md) | broken `${}` holding comments indents to the placeholder. See apps/oxfmt/DIVERGENCES.md#broken-template-comment-indent |

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
| [externals/ng-zorro-antd/components/style/themes/compact.less](diffs/less/externals__ng-zorro-antd__components__style__themes__compact.less.md) | fill break position (Prettier breaks inside the wide chunk, ours at the separator). See crates/oxc_formatter_css/DIVERGENCES.md#fill-break-position |
| [externals/ng-zorro-antd/components/style/themes/dark.less](diffs/less/externals__ng-zorro-antd__components__style__themes__dark.less.md) | trailing `//` comment never counts toward print width. See crates/oxc_formatter_css/DIVERGENCES.md#trailing-line-comment-print-width |
| [externals/ng-zorro-antd/components/style/themes/default.less](diffs/less/externals__ng-zorro-antd__components__style__themes__default.less.md) | fill break position (Prettier breaks inside the wide chunk, ours at the separator). See crates/oxc_formatter_css/DIVERGENCES.md#fill-break-position<br>trailing `//` comment never counts toward print width. See crates/oxc_formatter_css/DIVERGENCES.md#trailing-line-comment-print-width |
| [externals/ng-zorro-antd/components/style/themes/variable.less](diffs/less/externals__ng-zorro-antd__components__style__themes__variable.less.md) | fill break position (Prettier breaks inside the wide chunk, ours at the separator). See crates/oxc_formatter_css/DIVERGENCES.md#fill-break-position<br>trailing `//` comment never counts toward print width. See crates/oxc_formatter_css/DIVERGENCES.md#trailing-line-comment-print-width |
| [externals/ng-zorro-antd/components/table/style/index.less](diffs/less/externals__ng-zorro-antd__components__table__style__index.less.md) | fill break position (Prettier breaks inside the wide chunk, ours at the separator). See crates/oxc_formatter_css/DIVERGENCES.md#fill-break-position |
| [externals/ng-zorro-antd/components/table/style/rtl.less](diffs/less/externals__ng-zorro-antd__components__table__style__rtl.less.md) | fill break position (Prettier breaks inside the wide chunk, ours at the separator). See crates/oxc_formatter_css/DIVERGENCES.md#fill-break-position |

### Option 2: 406/409 (99.27%)

```json
{"printWidth":100}
```

| File | Note |
| :--- | :--- |
| [externals/ng-zorro-antd/components/style/themes/default.less](diffs/less/externals__ng-zorro-antd__components__style__themes__default.less.md) | fill break position (Prettier breaks inside the wide chunk, ours at the separator). See crates/oxc_formatter_css/DIVERGENCES.md#fill-break-position<br>trailing `//` comment never counts toward print width. See crates/oxc_formatter_css/DIVERGENCES.md#trailing-line-comment-print-width |
| [externals/ng-zorro-antd/components/style/themes/variable.less](diffs/less/externals__ng-zorro-antd__components__style__themes__variable.less.md) | fill break position (Prettier breaks inside the wide chunk, ours at the separator). See crates/oxc_formatter_css/DIVERGENCES.md#fill-break-position<br>trailing `//` comment never counts toward print width. See crates/oxc_formatter_css/DIVERGENCES.md#trailing-line-comment-print-width |
| [externals/ng-zorro-antd/components/table/style/rtl.less](diffs/less/externals__ng-zorro-antd__components__table__style__rtl.less.md) | fill break position (Prettier breaks inside the wide chunk, ours at the separator). See crates/oxc_formatter_css/DIVERGENCES.md#fill-break-position |

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

### Option 1: 295/302 (97.68%)

```json
{"printWidth":80}
```

| File | Note |
| :--- | :--- |
| [externals/aws-cloudformation-templates/ElasticLoadBalancing/ELB_Access_Logs_And_Connection_Draining.yaml](diffs/yaml/externals__aws-cloudformation-templates__ElasticLoadBalancing__ELB_Access_Logs_And_Connection_Draining.yaml.md) | block scalar trailing whitespace is part of the value. See crates/oxc_formatter_yaml/DIVERGENCES.md#block-scalar-trailing-whitespace |
| [externals/aws-cloudformation-templates/ElasticLoadBalancing/ELBGuidedAutoScalingRollingUpgrade.yaml](diffs/yaml/externals__aws-cloudformation-templates__ElasticLoadBalancing__ELBGuidedAutoScalingRollingUpgrade.yaml.md) | block scalar trailing whitespace is part of the value. See crates/oxc_formatter_yaml/DIVERGENCES.md#block-scalar-trailing-whitespace |
| [externals/aws-cloudformation-templates/ElasticLoadBalancing/ELBStickinessSample.yaml](diffs/yaml/externals__aws-cloudformation-templates__ElasticLoadBalancing__ELBStickinessSample.yaml.md) | block scalar trailing whitespace is part of the value. See crates/oxc_formatter_yaml/DIVERGENCES.md#block-scalar-trailing-whitespace |
| [externals/aws-cloudformation-templates/ElasticLoadBalancing/ELBWithLockedDownAutoScaledInstances.yaml](diffs/yaml/externals__aws-cloudformation-templates__ElasticLoadBalancing__ELBWithLockedDownAutoScaledInstances.yaml.md) | block scalar trailing whitespace is part of the value. See crates/oxc_formatter_yaml/DIVERGENCES.md#block-scalar-trailing-whitespace |
| [externals/aws-cloudformation-templates/RainModules/bucket.yml](diffs/yaml/externals__aws-cloudformation-templates__RainModules__bucket.yml.md) | block scalar trailing whitespace is part of the value. See crates/oxc_formatter_yaml/DIVERGENCES.md#block-scalar-trailing-whitespace |
| [externals/aws-cloudformation-templates/RainModules/load-balancer.yml](diffs/yaml/externals__aws-cloudformation-templates__RainModules__load-balancer.yml.md) | over-indented comment after `key: value` never rewrites the pair. See crates/oxc_formatter_yaml/DIVERGENCES.md#comment-over-indented |
| [externals/aws-cloudformation-templates/Solutions/OperatingSystems/ubuntu20.04_cfn-hup.yaml](diffs/yaml/externals__aws-cloudformation-templates__Solutions__OperatingSystems__ubuntu20.04_cfn-hup.yaml.md) | block scalar trailing whitespace is part of the value. See crates/oxc_formatter_yaml/DIVERGENCES.md#block-scalar-trailing-whitespace |

### Option 2: 295/302 (97.68%)

```json
{"printWidth":100,"tabWidth":4,"proseWrap":"always"}
```

| File | Note |
| :--- | :--- |
| [externals/aws-cloudformation-templates/ElasticLoadBalancing/ELB_Access_Logs_And_Connection_Draining.yaml](diffs/yaml/externals__aws-cloudformation-templates__ElasticLoadBalancing__ELB_Access_Logs_And_Connection_Draining.yaml.md) | block scalar trailing whitespace is part of the value. See crates/oxc_formatter_yaml/DIVERGENCES.md#block-scalar-trailing-whitespace |
| [externals/aws-cloudformation-templates/ElasticLoadBalancing/ELBGuidedAutoScalingRollingUpgrade.yaml](diffs/yaml/externals__aws-cloudformation-templates__ElasticLoadBalancing__ELBGuidedAutoScalingRollingUpgrade.yaml.md) | block scalar trailing whitespace is part of the value. See crates/oxc_formatter_yaml/DIVERGENCES.md#block-scalar-trailing-whitespace |
| [externals/aws-cloudformation-templates/ElasticLoadBalancing/ELBStickinessSample.yaml](diffs/yaml/externals__aws-cloudformation-templates__ElasticLoadBalancing__ELBStickinessSample.yaml.md) | block scalar trailing whitespace is part of the value. See crates/oxc_formatter_yaml/DIVERGENCES.md#block-scalar-trailing-whitespace |
| [externals/aws-cloudformation-templates/ElasticLoadBalancing/ELBWithLockedDownAutoScaledInstances.yaml](diffs/yaml/externals__aws-cloudformation-templates__ElasticLoadBalancing__ELBWithLockedDownAutoScaledInstances.yaml.md) | block scalar trailing whitespace is part of the value. See crates/oxc_formatter_yaml/DIVERGENCES.md#block-scalar-trailing-whitespace |
| [externals/aws-cloudformation-templates/RainModules/bucket.yml](diffs/yaml/externals__aws-cloudformation-templates__RainModules__bucket.yml.md) | block scalar trailing whitespace is part of the value. See crates/oxc_formatter_yaml/DIVERGENCES.md#block-scalar-trailing-whitespace |
| [externals/aws-cloudformation-templates/RainModules/load-balancer.yml](diffs/yaml/externals__aws-cloudformation-templates__RainModules__load-balancer.yml.md) | over-indented comment after `key: value` never rewrites the pair. See crates/oxc_formatter_yaml/DIVERGENCES.md#comment-over-indented |
| [externals/aws-cloudformation-templates/Solutions/OperatingSystems/ubuntu20.04_cfn-hup.yaml](diffs/yaml/externals__aws-cloudformation-templates__Solutions__OperatingSystems__ubuntu20.04_cfn-hup.yaml.md) | block scalar trailing whitespace is part of the value. See crates/oxc_formatter_yaml/DIVERGENCES.md#block-scalar-trailing-whitespace |

### Option 3: 295/302 (97.68%)

```json
{"printWidth":120,"singleQuote":true,"bracketSpacing":false,"trailingComma":"none"}
```

| File | Note |
| :--- | :--- |
| [externals/aws-cloudformation-templates/ElasticLoadBalancing/ELB_Access_Logs_And_Connection_Draining.yaml](diffs/yaml/externals__aws-cloudformation-templates__ElasticLoadBalancing__ELB_Access_Logs_And_Connection_Draining.yaml.md) | block scalar trailing whitespace is part of the value. See crates/oxc_formatter_yaml/DIVERGENCES.md#block-scalar-trailing-whitespace |
| [externals/aws-cloudformation-templates/ElasticLoadBalancing/ELBGuidedAutoScalingRollingUpgrade.yaml](diffs/yaml/externals__aws-cloudformation-templates__ElasticLoadBalancing__ELBGuidedAutoScalingRollingUpgrade.yaml.md) | block scalar trailing whitespace is part of the value. See crates/oxc_formatter_yaml/DIVERGENCES.md#block-scalar-trailing-whitespace |
| [externals/aws-cloudformation-templates/ElasticLoadBalancing/ELBStickinessSample.yaml](diffs/yaml/externals__aws-cloudformation-templates__ElasticLoadBalancing__ELBStickinessSample.yaml.md) | block scalar trailing whitespace is part of the value. See crates/oxc_formatter_yaml/DIVERGENCES.md#block-scalar-trailing-whitespace |
| [externals/aws-cloudformation-templates/ElasticLoadBalancing/ELBWithLockedDownAutoScaledInstances.yaml](diffs/yaml/externals__aws-cloudformation-templates__ElasticLoadBalancing__ELBWithLockedDownAutoScaledInstances.yaml.md) | block scalar trailing whitespace is part of the value. See crates/oxc_formatter_yaml/DIVERGENCES.md#block-scalar-trailing-whitespace |
| [externals/aws-cloudformation-templates/RainModules/bucket.yml](diffs/yaml/externals__aws-cloudformation-templates__RainModules__bucket.yml.md) | block scalar trailing whitespace is part of the value. See crates/oxc_formatter_yaml/DIVERGENCES.md#block-scalar-trailing-whitespace |
| [externals/aws-cloudformation-templates/RainModules/load-balancer.yml](diffs/yaml/externals__aws-cloudformation-templates__RainModules__load-balancer.yml.md) | over-indented comment after `key: value` never rewrites the pair. See crates/oxc_formatter_yaml/DIVERGENCES.md#comment-over-indented |
| [externals/aws-cloudformation-templates/Solutions/OperatingSystems/ubuntu20.04_cfn-hup.yaml](diffs/yaml/externals__aws-cloudformation-templates__Solutions__OperatingSystems__ubuntu20.04_cfn-hup.yaml.md) | block scalar trailing whitespace is part of the value. See crates/oxc_formatter_yaml/DIVERGENCES.md#block-scalar-trailing-whitespace |

## scss

### Option 1: 203/217 (93.55%)

```json
{"printWidth":80}
```

| File | Note |
| :--- | :--- |
| [externals/gitlab/stylesheets/components/content_editor.scss](diffs/scss/externals__gitlab__stylesheets__components__content_editor.scss.md) | fill break position (Prettier breaks inside the wide chunk, ours at the separator). See crates/oxc_formatter_css/DIVERGENCES.md#fill-break-position |
| [externals/gitlab/stylesheets/framework/diffs.scss](diffs/scss/externals__gitlab__stylesheets__framework__diffs.scss.md) | media-query operator spacing. See crates/oxc_formatter_css/DIVERGENCES.md#media-query-operator-spacing |
| [externals/gitlab/stylesheets/framework/variables_overrides.scss](diffs/scss/externals__gitlab__stylesheets__framework__variables_overrides.scss.md) | no trailing comma into non-comma-list map-item parens. See crates/oxc_formatter_css/DIVERGENCES.md#map-item-break-comma-lists-only |
| [externals/gitlab/stylesheets/highlight/conflict_colors.scss](diffs/scss/externals__gitlab__stylesheets__highlight__conflict_colors.scss.md) | blank lines in maps with paren values are preserved. See crates/oxc_formatter_css/DIVERGENCES.md#map-paren-value-blank-lines |
| [externals/gitlab/stylesheets/page_bundles/_ide_theme_overrides.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles___ide_theme_overrides.scss.md) | fill break position (Prettier breaks inside the wide chunk, ours at the separator). See crates/oxc_formatter_css/DIVERGENCES.md#fill-break-position |
| [externals/gitlab/stylesheets/page_bundles/editor.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__editor.scss.md) | media-query operator spacing. See crates/oxc_formatter_css/DIVERGENCES.md#media-query-operator-spacing |
| [externals/gitlab/stylesheets/page_bundles/environments.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__environments.scss.md) | media-query operator spacing. See crates/oxc_formatter_css/DIVERGENCES.md#media-query-operator-spacing |
| [externals/gitlab/stylesheets/page_bundles/issuable_list.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__issuable_list.scss.md) | media-query operator spacing. See crates/oxc_formatter_css/DIVERGENCES.md#media-query-operator-spacing |
| [externals/gitlab/stylesheets/page_bundles/labels.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__labels.scss.md) | media-query operator spacing. See crates/oxc_formatter_css/DIVERGENCES.md#media-query-operator-spacing |
| [externals/gitlab/stylesheets/page_bundles/merge_requests.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__merge_requests.scss.md) | media-query operator spacing. See crates/oxc_formatter_css/DIVERGENCES.md#media-query-operator-spacing |
| [externals/gitlab/stylesheets/page_bundles/projects.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__projects.scss.md) | media-query operator spacing. See crates/oxc_formatter_css/DIVERGENCES.md#media-query-operator-spacing |
| [externals/gitlab/stylesheets/page_bundles/settings.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__settings.scss.md) | media-query operator spacing. See crates/oxc_formatter_css/DIVERGENCES.md#media-query-operator-spacing |
| [externals/gitlab/stylesheets/pages/profile.scss](diffs/scss/externals__gitlab__stylesheets__pages__profile.scss.md) | trailing `//` comment never counts toward print width. See crates/oxc_formatter_css/DIVERGENCES.md#trailing-line-comment-print-width |
| [externals/gitlab/stylesheets/pages/settings.scss](diffs/scss/externals__gitlab__stylesheets__pages__settings.scss.md) | media-query operator spacing. See crates/oxc_formatter_css/DIVERGENCES.md#media-query-operator-spacing |

### Option 2: 204/217 (94.01%)

```json
{"printWidth":100}
```

| File | Note |
| :--- | :--- |
| [externals/gitlab/stylesheets/framework/diffs.scss](diffs/scss/externals__gitlab__stylesheets__framework__diffs.scss.md) | media-query operator spacing. See crates/oxc_formatter_css/DIVERGENCES.md#media-query-operator-spacing |
| [externals/gitlab/stylesheets/framework/sidebar.scss](diffs/scss/externals__gitlab__stylesheets__framework__sidebar.scss.md) | fill break position (Prettier breaks inside the wide chunk, ours at the separator). See crates/oxc_formatter_css/DIVERGENCES.md#fill-break-position |
| [externals/gitlab/stylesheets/framework/variables_overrides.scss](diffs/scss/externals__gitlab__stylesheets__framework__variables_overrides.scss.md) | no trailing comma into non-comma-list map-item parens. See crates/oxc_formatter_css/DIVERGENCES.md#map-item-break-comma-lists-only |
| [externals/gitlab/stylesheets/highlight/conflict_colors.scss](diffs/scss/externals__gitlab__stylesheets__highlight__conflict_colors.scss.md) | blank lines in maps with paren values are preserved. See crates/oxc_formatter_css/DIVERGENCES.md#map-paren-value-blank-lines |
| [externals/gitlab/stylesheets/page_bundles/_ide_theme_overrides.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles___ide_theme_overrides.scss.md) | fill break position (Prettier breaks inside the wide chunk, ours at the separator). See crates/oxc_formatter_css/DIVERGENCES.md#fill-break-position |
| [externals/gitlab/stylesheets/page_bundles/editor.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__editor.scss.md) | media-query operator spacing. See crates/oxc_formatter_css/DIVERGENCES.md#media-query-operator-spacing |
| [externals/gitlab/stylesheets/page_bundles/environments.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__environments.scss.md) | media-query operator spacing. See crates/oxc_formatter_css/DIVERGENCES.md#media-query-operator-spacing |
| [externals/gitlab/stylesheets/page_bundles/issuable_list.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__issuable_list.scss.md) | media-query operator spacing. See crates/oxc_formatter_css/DIVERGENCES.md#media-query-operator-spacing |
| [externals/gitlab/stylesheets/page_bundles/labels.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__labels.scss.md) | media-query operator spacing. See crates/oxc_formatter_css/DIVERGENCES.md#media-query-operator-spacing |
| [externals/gitlab/stylesheets/page_bundles/merge_requests.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__merge_requests.scss.md) | media-query operator spacing. See crates/oxc_formatter_css/DIVERGENCES.md#media-query-operator-spacing |
| [externals/gitlab/stylesheets/page_bundles/projects.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__projects.scss.md) | media-query operator spacing. See crates/oxc_formatter_css/DIVERGENCES.md#media-query-operator-spacing |
| [externals/gitlab/stylesheets/page_bundles/settings.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__settings.scss.md) | media-query operator spacing. See crates/oxc_formatter_css/DIVERGENCES.md#media-query-operator-spacing |
| [externals/gitlab/stylesheets/pages/settings.scss](diffs/scss/externals__gitlab__stylesheets__pages__settings.scss.md) | media-query operator spacing. See crates/oxc_formatter_css/DIVERGENCES.md#media-query-operator-spacing |
