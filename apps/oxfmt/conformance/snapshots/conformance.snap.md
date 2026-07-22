## js-in-vue

### Option 1: 422/425 (99.29%)

```json
{"printWidth":80}
```

| File | Note |
| :--- | :--- |
| [externals/prettier/vue/multiparser/lang-tsx.vue](diffs/js-in-vue/externals__prettier__vue__multiparser__lang-tsx.vue.md) | `lang=tsx` is not supported |
| [externals/vue-vben-admin/@core/ui-kit/shadcn-ui/src/components/render-content/render-content.vue](diffs/js-in-vue/externals__vue-vben-admin__@core__ui-kit__shadcn-ui__src__components__render-content__render-content.vue.md) |  |
| [externals/vue-vben-admin/effects/common-ui/src/components/api-component/api-component.vue](diffs/js-in-vue/externals__vue-vben-admin__effects__common-ui__src__components__api-component__api-component.vue.md) | `<T = any,>() => {}` comma in generic param is removed even in .ts(x) file |

### Option 2: 423/425 (99.53%)

```json
{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true}
```

| File | Note |
| :--- | :--- |
| [externals/prettier/vue/multiparser/lang-tsx.vue](diffs/js-in-vue/externals__prettier__vue__multiparser__lang-tsx.vue.md) | `lang=tsx` is not supported |
| [externals/vue-vben-admin/effects/common-ui/src/components/api-component/api-component.vue](diffs/js-in-vue/externals__vue-vben-admin__effects__common-ui__src__components__api-component__api-component.vue.md) | `<T = any,>() => {}` comma in generic param is removed even in .ts(x) file |

## gql-in-js

### Option 1: 12/12 (100.00%)

```json
{"printWidth":80}
```

### Option 2: 12/12 (100.00%)

```json
{"printWidth":100}
```

## css-in-js

### Option 1: 18/20 (90.00%)

```json
{"printWidth":80}
```

| File | Note |
| :--- | :--- |
| [externals/prettier/js/multiparser-css/issue-5697.js](diffs/css-in-js/externals__prettier__js__multiparser-css__issue-5697.js.md) |  |
| [externals/prettier/js/multiparser-css/styled-components.js](diffs/css-in-js/externals__prettier__js__multiparser-css__styled-components.js.md) | `Xxx.extend` not recognized as tag |

### Option 2: 18/20 (90.00%)

```json
{"printWidth":100}
```

| File | Note |
| :--- | :--- |
| [externals/prettier/js/multiparser-css/issue-5697.js](diffs/css-in-js/externals__prettier__js__multiparser-css__issue-5697.js.md) |  |
| [externals/prettier/js/multiparser-css/styled-components.js](diffs/css-in-js/externals__prettier__js__multiparser-css__styled-components.js.md) | `Xxx.extend` not recognized as tag |

## html-in-js

### Option 1: 149/191 (78.01%)

```json
{"printWidth":80}
```

| File | Note |
| :--- | :--- |
| [externals/prettier/js/multiparser-html/issue-10691.js](diffs/html-in-js/externals__prettier__js__multiparser-html__issue-10691.js.md) | js-in-html(`<script>`)-in-js needs lot more work; Please see oxc_formatter/src/print/template/embed/html.rs |
| [externals/webawesome/animated-image/animated-image.ts](diffs/html-in-js/externals__webawesome__animated-image__animated-image.ts.md) |  |
| [externals/webawesome/badge/badge.ts](diffs/html-in-js/externals__webawesome__badge__badge.ts.md) |  |
| [externals/webawesome/breadcrumb-item/breadcrumb-item.ts](diffs/html-in-js/externals__webawesome__breadcrumb-item__breadcrumb-item.ts.md) |  |
| [externals/webawesome/breadcrumb/breadcrumb.ts](diffs/html-in-js/externals__webawesome__breadcrumb__breadcrumb.ts.md) |  |
| [externals/webawesome/button/button.ts](diffs/html-in-js/externals__webawesome__button__button.ts.md) |  |
| [externals/webawesome/callout/callout.ts](diffs/html-in-js/externals__webawesome__callout__callout.ts.md) |  |
| [externals/webawesome/card/card.ts](diffs/html-in-js/externals__webawesome__card__card.ts.md) |  |
| [externals/webawesome/carousel/carousel.test.ts](diffs/html-in-js/externals__webawesome__carousel__carousel.test.ts.md) |  |
| [externals/webawesome/carousel/carousel.ts](diffs/html-in-js/externals__webawesome__carousel__carousel.ts.md) |  |
| [externals/webawesome/checkbox/checkbox.test.ts](diffs/html-in-js/externals__webawesome__checkbox__checkbox.test.ts.md) |  |
| [externals/webawesome/checkbox/checkbox.ts](diffs/html-in-js/externals__webawesome__checkbox__checkbox.ts.md) |  |
| [externals/webawesome/color-picker/color-picker.ts](diffs/html-in-js/externals__webawesome__color-picker__color-picker.ts.md) |  |
| [externals/webawesome/copy-button/copy-button.ts](diffs/html-in-js/externals__webawesome__copy-button__copy-button.ts.md) |  |
| [externals/webawesome/details/details.ts](diffs/html-in-js/externals__webawesome__details__details.ts.md) |  |
| [externals/webawesome/dialog/dialog.ts](diffs/html-in-js/externals__webawesome__dialog__dialog.ts.md) |  |
| [externals/webawesome/drawer/drawer.ts](diffs/html-in-js/externals__webawesome__drawer__drawer.ts.md) |  |
| [externals/webawesome/dropdown-item/dropdown-item.ts](diffs/html-in-js/externals__webawesome__dropdown-item__dropdown-item.ts.md) |  |
| [externals/webawesome/dropdown/dropdown.ts](diffs/html-in-js/externals__webawesome__dropdown__dropdown.ts.md) |  |
| [externals/webawesome/format-number/format-number.ts](diffs/html-in-js/externals__webawesome__format-number__format-number.ts.md) |  |
| [externals/webawesome/input/input.ts](diffs/html-in-js/externals__webawesome__input__input.ts.md) |  |
| [externals/webawesome/number-input/number-input.styles.ts](diffs/html-in-js/externals__webawesome__number-input__number-input.styles.ts.md) | Layout-only: Prettier's fill fit-check breaks inside `var()` args in a long `calc()`; ours breaks after the operator. See crates/oxc_formatter_css/AGENTS.md |
| [externals/webawesome/number-input/number-input.ts](diffs/html-in-js/externals__webawesome__number-input__number-input.ts.md) |  |
| [externals/webawesome/option/option.ts](diffs/html-in-js/externals__webawesome__option__option.ts.md) |  |
| [externals/webawesome/page/page.styles.ts](diffs/html-in-js/externals__webawesome__page__page.styles.ts.md) | Layout-only: Prettier's fill fit-check breaks inside `::slotted()` after a long `:not(...)`; ours breaks inside `:not(...)`. See crates/oxc_formatter_css/AGENTS.md |
| [externals/webawesome/page/page.ts](diffs/html-in-js/externals__webawesome__page__page.ts.md) |  |
| [externals/webawesome/popup/popup.ts](diffs/html-in-js/externals__webawesome__popup__popup.ts.md) |  |
| [externals/webawesome/progress-bar/progress-bar.ts](diffs/html-in-js/externals__webawesome__progress-bar__progress-bar.ts.md) |  |
| [externals/webawesome/progress-ring/progress-ring.ts](diffs/html-in-js/externals__webawesome__progress-ring__progress-ring.ts.md) |  |
| [externals/webawesome/qr-code/qr-code.ts](diffs/html-in-js/externals__webawesome__qr-code__qr-code.ts.md) |  |
| [externals/webawesome/radio-group/radio-group.ts](diffs/html-in-js/externals__webawesome__radio-group__radio-group.ts.md) |  |
| [externals/webawesome/radio/radio.ts](diffs/html-in-js/externals__webawesome__radio__radio.ts.md) |  |
| [externals/webawesome/rating/rating.ts](diffs/html-in-js/externals__webawesome__rating__rating.ts.md) |  |
| [externals/webawesome/scroller/scroller.ts](diffs/html-in-js/externals__webawesome__scroller__scroller.ts.md) |  |
| [externals/webawesome/select/select.ts](diffs/html-in-js/externals__webawesome__select__select.ts.md) |  |
| [externals/webawesome/slider/slider.ts](diffs/html-in-js/externals__webawesome__slider__slider.ts.md) |  |
| [externals/webawesome/switch/switch.test.ts](diffs/html-in-js/externals__webawesome__switch__switch.test.ts.md) |  |
| [externals/webawesome/switch/switch.ts](diffs/html-in-js/externals__webawesome__switch__switch.ts.md) |  |
| [externals/webawesome/tab-group/tab-group.ts](diffs/html-in-js/externals__webawesome__tab-group__tab-group.ts.md) |  |
| [externals/webawesome/tag/tag.ts](diffs/html-in-js/externals__webawesome__tag__tag.ts.md) |  |
| [externals/webawesome/textarea/textarea.ts](diffs/html-in-js/externals__webawesome__textarea__textarea.ts.md) |  |
| [externals/webawesome/zoomable-frame/zoomable-frame.ts](diffs/html-in-js/externals__webawesome__zoomable-frame__zoomable-frame.ts.md) |  |

### Option 2: 161/191 (84.29%)

```json
{"printWidth":100,"htmlWhitespaceSensitivity":"ignore"}
```

| File | Note |
| :--- | :--- |
| [externals/prettier/js/multiparser-html/issue-10691.js](diffs/html-in-js/externals__prettier__js__multiparser-html__issue-10691.js.md) | js-in-html(`<script>`)-in-js needs lot more work; Please see oxc_formatter/src/print/template/embed/html.rs |
| [externals/webawesome/animated-image/animated-image.ts](diffs/html-in-js/externals__webawesome__animated-image__animated-image.ts.md) |  |
| [externals/webawesome/breadcrumb-item/breadcrumb-item.ts](diffs/html-in-js/externals__webawesome__breadcrumb-item__breadcrumb-item.ts.md) |  |
| [externals/webawesome/button/button.ts](diffs/html-in-js/externals__webawesome__button__button.ts.md) |  |
| [externals/webawesome/callout/callout.ts](diffs/html-in-js/externals__webawesome__callout__callout.ts.md) |  |
| [externals/webawesome/card/card.ts](diffs/html-in-js/externals__webawesome__card__card.ts.md) |  |
| [externals/webawesome/carousel/carousel.ts](diffs/html-in-js/externals__webawesome__carousel__carousel.ts.md) |  |
| [externals/webawesome/checkbox/checkbox.ts](diffs/html-in-js/externals__webawesome__checkbox__checkbox.ts.md) |  |
| [externals/webawesome/color-picker/color-picker.ts](diffs/html-in-js/externals__webawesome__color-picker__color-picker.ts.md) |  |
| [externals/webawesome/copy-button/copy-button.ts](diffs/html-in-js/externals__webawesome__copy-button__copy-button.ts.md) |  |
| [externals/webawesome/dialog/dialog.ts](diffs/html-in-js/externals__webawesome__dialog__dialog.ts.md) |  |
| [externals/webawesome/drawer/drawer.ts](diffs/html-in-js/externals__webawesome__drawer__drawer.ts.md) |  |
| [externals/webawesome/dropdown-item/dropdown-item.ts](diffs/html-in-js/externals__webawesome__dropdown-item__dropdown-item.ts.md) |  |
| [externals/webawesome/format-number/format-number.ts](diffs/html-in-js/externals__webawesome__format-number__format-number.ts.md) |  |
| [externals/webawesome/icon/icon.ts](diffs/html-in-js/externals__webawesome__icon__icon.ts.md) |  |
| [externals/webawesome/input/input.ts](diffs/html-in-js/externals__webawesome__input__input.ts.md) |  |
| [externals/webawesome/number-input/number-input.ts](diffs/html-in-js/externals__webawesome__number-input__number-input.ts.md) |  |
| [externals/webawesome/option/option.ts](diffs/html-in-js/externals__webawesome__option__option.ts.md) |  |
| [externals/webawesome/page/page.ts](diffs/html-in-js/externals__webawesome__page__page.ts.md) |  |
| [externals/webawesome/popup/popup.ts](diffs/html-in-js/externals__webawesome__popup__popup.ts.md) |  |
| [externals/webawesome/progress-bar/progress-bar.ts](diffs/html-in-js/externals__webawesome__progress-bar__progress-bar.ts.md) |  |
| [externals/webawesome/radio/radio.ts](diffs/html-in-js/externals__webawesome__radio__radio.ts.md) |  |
| [externals/webawesome/scroller/scroller.ts](diffs/html-in-js/externals__webawesome__scroller__scroller.ts.md) |  |
| [externals/webawesome/select/select.ts](diffs/html-in-js/externals__webawesome__select__select.ts.md) |  |
| [externals/webawesome/slider/slider.ts](diffs/html-in-js/externals__webawesome__slider__slider.ts.md) |  |
| [externals/webawesome/switch/switch.ts](diffs/html-in-js/externals__webawesome__switch__switch.ts.md) |  |
| [externals/webawesome/tab-group/tab-group.ts](diffs/html-in-js/externals__webawesome__tab-group__tab-group.ts.md) |  |
| [externals/webawesome/tag/tag.ts](diffs/html-in-js/externals__webawesome__tag__tag.ts.md) |  |
| [externals/webawesome/textarea/textarea.ts](diffs/html-in-js/externals__webawesome__textarea__textarea.ts.md) |  |
| [externals/webawesome/zoomable-frame/zoomable-frame.ts](diffs/html-in-js/externals__webawesome__zoomable-frame__zoomable-frame.ts.md) |  |

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
| [externals/prettier/js/multiparser-comments/comment-inside.js](diffs/xxx-in-js-comment/externals__prettier__js__multiparser-comments__comment-inside.js.md) |  |

### Option 2: 4/5 (80.00%)

```json
{"printWith":100}
```

| File | Note |
| :--- | :--- |
| [externals/prettier/js/multiparser-comments/comment-inside.js](diffs/xxx-in-js-comment/externals__prettier__js__multiparser-comments__comment-inside.js.md) |  |

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

### Option 1: 394/409 (96.33%)

```json
{"printWidth":80}
```

| File | Note |
| :--- | :--- |
| [externals/ng-zorro-antd/components/button/style/space-compact.less](diffs/less/externals__ng-zorro-antd__components__button__style__space-compact.less.md) | Allowed (layout-only): wrapped :not() selector-arg indent (prettier/prettier#16165) |
| [externals/ng-zorro-antd/components/date-picker/style/panel.less](diffs/less/externals__ng-zorro-antd__components__date-picker__style__panel.less.md) | Allowed (layout-only): wrapped :not() selector-arg indent (prettier/prettier#16165) |
| [externals/ng-zorro-antd/components/date-picker/style/rtl.less](diffs/less/externals__ng-zorro-antd__components__date-picker__style__rtl.less.md) | Allowed (layout-only): wrapped :not() selector-arg indent (prettier/prettier#16165) |
| [externals/ng-zorro-antd/components/form/style/index.less](diffs/less/externals__ng-zorro-antd__components__form__style__index.less.md) | Allowed (layout-only): wrapped :not() selector-arg indent (prettier/prettier#16165) |
| [externals/ng-zorro-antd/components/input/style/mixin.less](diffs/less/externals__ng-zorro-antd__components__input__style__mixin.less.md) | Allowed (layout-only): wrapped :not() selector-arg indent (prettier/prettier#16165) |
| [externals/ng-zorro-antd/components/mention/style/patch.less](diffs/less/externals__ng-zorro-antd__components__mention__style__patch.less.md) | Allowed (layout-only): wrapped :not() selector-arg indent (prettier/prettier#16165) |
| [externals/ng-zorro-antd/components/radio/style/rtl.less](diffs/less/externals__ng-zorro-antd__components__radio__style__rtl.less.md) | Allowed (layout-only): wrapped :not() selector-arg indent (prettier/prettier#16165) |
| [externals/ng-zorro-antd/components/select/style/status.less](diffs/less/externals__ng-zorro-antd__components__select__style__status.less.md) | Allowed (layout-only): wrapped :not() selector-arg indent (prettier/prettier#16165) |
| [externals/ng-zorro-antd/components/style/mixins/customize.less](diffs/less/externals__ng-zorro-antd__components__style__mixins__customize.less.md) | Allowed (layout-only): wrapped :not() selector-arg indent (prettier/prettier#16165) |
| [externals/ng-zorro-antd/components/style/themes/compact.less](diffs/less/externals__ng-zorro-antd__components__style__themes__compact.less.md) | Allowed (layout-only): nested Less math — Prettier's fill fit-check breaks inside the wide chunk, ours breaks the separator (biome fill). See crates/oxc_formatter_css/AGENTS.md |
| [externals/ng-zorro-antd/components/style/themes/dark.less](diffs/less/externals__ng-zorro-antd__components__style__themes__dark.less.md) | Allowed: trailing `//` comment doesn't count toward print width, so the value stays flat where Prettier breaks it. |
| [externals/ng-zorro-antd/components/style/themes/default.less](diffs/less/externals__ng-zorro-antd__components__style__themes__default.less.md) | Allowed (layout-only): nested Less math — Prettier's fill fit-check breaks inside the wide chunk, ours breaks the separator (biome fill). See crates/oxc_formatter_css/AGENTS.md<br>Allowed: trailing `//` comment doesn't count toward print width, so the value stays flat where Prettier breaks it. |
| [externals/ng-zorro-antd/components/style/themes/variable.less](diffs/less/externals__ng-zorro-antd__components__style__themes__variable.less.md) | Allowed (layout-only): nested Less math — Prettier's fill fit-check breaks inside the wide chunk, ours breaks the separator (biome fill). See crates/oxc_formatter_css/AGENTS.md<br>Allowed: trailing `//` comment doesn't count toward print width, so the value stays flat where Prettier breaks it. |
| [externals/ng-zorro-antd/components/table/style/index.less](diffs/less/externals__ng-zorro-antd__components__table__style__index.less.md) | Allowed (layout-only): wrapped :not() selector-arg indent (prettier/prettier#16165)<br>Allowed (layout-only): nested Less math — Prettier's fill fit-check breaks inside the wide chunk, ours breaks the separator (biome fill). See crates/oxc_formatter_css/AGENTS.md |
| [externals/ng-zorro-antd/components/table/style/rtl.less](diffs/less/externals__ng-zorro-antd__components__table__style__rtl.less.md) | Allowed (layout-only): wrapped :not() selector-arg indent (prettier/prettier#16165)<br>Allowed (layout-only): nested Less math — Prettier's fill fit-check breaks inside the wide chunk, ours breaks the separator (biome fill). See crates/oxc_formatter_css/AGENTS.md |

### Option 2: 399/409 (97.56%)

```json
{"printWidth":100}
```

| File | Note |
| :--- | :--- |
| [externals/ng-zorro-antd/components/date-picker/style/panel.less](diffs/less/externals__ng-zorro-antd__components__date-picker__style__panel.less.md) | Allowed (layout-only): wrapped :not() selector-arg indent (prettier/prettier#16165) |
| [externals/ng-zorro-antd/components/form/style/index.less](diffs/less/externals__ng-zorro-antd__components__form__style__index.less.md) | Allowed (layout-only): wrapped :not() selector-arg indent (prettier/prettier#16165) |
| [externals/ng-zorro-antd/components/input/style/mixin.less](diffs/less/externals__ng-zorro-antd__components__input__style__mixin.less.md) | Allowed (layout-only): wrapped :not() selector-arg indent (prettier/prettier#16165) |
| [externals/ng-zorro-antd/components/mention/style/patch.less](diffs/less/externals__ng-zorro-antd__components__mention__style__patch.less.md) | Allowed (layout-only): wrapped :not() selector-arg indent (prettier/prettier#16165) |
| [externals/ng-zorro-antd/components/radio/style/rtl.less](diffs/less/externals__ng-zorro-antd__components__radio__style__rtl.less.md) | Allowed (layout-only): wrapped :not() selector-arg indent (prettier/prettier#16165) |
| [externals/ng-zorro-antd/components/select/style/status.less](diffs/less/externals__ng-zorro-antd__components__select__style__status.less.md) | Allowed (layout-only): wrapped :not() selector-arg indent (prettier/prettier#16165) |
| [externals/ng-zorro-antd/components/style/themes/default.less](diffs/less/externals__ng-zorro-antd__components__style__themes__default.less.md) | Allowed (layout-only): nested Less math — Prettier's fill fit-check breaks inside the wide chunk, ours breaks the separator (biome fill). See crates/oxc_formatter_css/AGENTS.md<br>Allowed: trailing `//` comment doesn't count toward print width, so the value stays flat where Prettier breaks it. |
| [externals/ng-zorro-antd/components/style/themes/variable.less](diffs/less/externals__ng-zorro-antd__components__style__themes__variable.less.md) | Allowed (layout-only): nested Less math — Prettier's fill fit-check breaks inside the wide chunk, ours breaks the separator (biome fill). See crates/oxc_formatter_css/AGENTS.md<br>Allowed: trailing `//` comment doesn't count toward print width, so the value stays flat where Prettier breaks it. |
| [externals/ng-zorro-antd/components/table/style/index.less](diffs/less/externals__ng-zorro-antd__components__table__style__index.less.md) | Allowed (layout-only): wrapped :not() selector-arg indent (prettier/prettier#16165)<br>Allowed (layout-only): nested Less math — Prettier's fill fit-check breaks inside the wide chunk, ours breaks the separator (biome fill). See crates/oxc_formatter_css/AGENTS.md |
| [externals/ng-zorro-antd/components/table/style/rtl.less](diffs/less/externals__ng-zorro-antd__components__table__style__rtl.less.md) | Allowed (layout-only): wrapped :not() selector-arg indent (prettier/prettier#16165)<br>Allowed (layout-only): nested Less math — Prettier's fill fit-check breaks inside the wide chunk, ours breaks the separator (biome fill). See crates/oxc_formatter_css/AGENTS.md |

## css

### Option 1: 221/221 (100.00%)

```json
{"printWidth":80}
```

### Option 2: 221/221 (100.00%)

```json
{"printWidth":100}
```

## scss

### Option 1: 201/217 (92.63%)

```json
{"printWidth":80}
```

| File | Note |
| :--- | :--- |
| [externals/gitlab/stylesheets/components/content_editor.scss](diffs/scss/externals__gitlab__stylesheets__components__content_editor.scss.md) |  |
| [externals/gitlab/stylesheets/framework/diffs.scss](diffs/scss/externals__gitlab__stylesheets__framework__diffs.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/framework/sidebar.scss](diffs/scss/externals__gitlab__stylesheets__framework__sidebar.scss.md) | Allowed (layout-only): wrapped :not() selector-arg indent (prettier/prettier#16165)<br>logn-expr line-break position |
| [externals/gitlab/stylesheets/framework/variables_overrides.scss](diffs/scss/externals__gitlab__stylesheets__framework__variables_overrides.scss.md) | Allowed (semantics): Prettier adds a trailing comma to non-comma-list map-item parens (`1: ($spacer * 0.5)` → 1-element list); we keep them inline. See crates/oxc_formatter_css/AGENTS.md |
| [externals/gitlab/stylesheets/highlight/conflict_colors.scss](diffs/scss/externals__gitlab__stylesheets__highlight__conflict_colors.scss.md) | Allowed: Prettier drops blank lines in SCSS maps with paren values; ours preserves (prettier/prettier#16824) |
| [externals/gitlab/stylesheets/highlight/white_base.scss](diffs/scss/externals__gitlab__stylesheets__highlight__white_base.scss.md) | Allowed (layout-only): wrapped :not() selector-arg indent (prettier/prettier#16165) |
| [externals/gitlab/stylesheets/page_bundles/_ide_theme_overrides.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles___ide_theme_overrides.scss.md) |  |
| [externals/gitlab/stylesheets/page_bundles/editor.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__editor.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/page_bundles/environments.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__environments.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/page_bundles/issuable_list.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__issuable_list.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/page_bundles/labels.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__labels.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/page_bundles/merge_requests.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__merge_requests.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/page_bundles/projects.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__projects.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/page_bundles/settings.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__settings.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/pages/profile.scss](diffs/scss/externals__gitlab__stylesheets__pages__profile.scss.md) | Allowed: trailing `//` comment doesn't count toward print width, so the value stays flat where Prettier breaks it. |
| [externals/gitlab/stylesheets/pages/settings.scss](diffs/scss/externals__gitlab__stylesheets__pages__settings.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |

### Option 2: 204/217 (94.01%)

```json
{"printWidth":100}
```

| File | Note |
| :--- | :--- |
| [externals/gitlab/stylesheets/framework/diffs.scss](diffs/scss/externals__gitlab__stylesheets__framework__diffs.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/framework/sidebar.scss](diffs/scss/externals__gitlab__stylesheets__framework__sidebar.scss.md) | Allowed (layout-only): wrapped :not() selector-arg indent (prettier/prettier#16165)<br>logn-expr line-break position |
| [externals/gitlab/stylesheets/framework/variables_overrides.scss](diffs/scss/externals__gitlab__stylesheets__framework__variables_overrides.scss.md) | Allowed (semantics): Prettier adds a trailing comma to non-comma-list map-item parens (`1: ($spacer * 0.5)` → 1-element list); we keep them inline. See crates/oxc_formatter_css/AGENTS.md |
| [externals/gitlab/stylesheets/highlight/conflict_colors.scss](diffs/scss/externals__gitlab__stylesheets__highlight__conflict_colors.scss.md) | Allowed: Prettier drops blank lines in SCSS maps with paren values; ours preserves (prettier/prettier#16824) |
| [externals/gitlab/stylesheets/page_bundles/_ide_theme_overrides.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles___ide_theme_overrides.scss.md) |  |
| [externals/gitlab/stylesheets/page_bundles/editor.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__editor.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/page_bundles/environments.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__environments.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/page_bundles/issuable_list.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__issuable_list.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/page_bundles/labels.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__labels.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/page_bundles/merge_requests.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__merge_requests.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/page_bundles/projects.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__projects.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/page_bundles/settings.scss](diffs/scss/externals__gitlab__stylesheets__page_bundles__settings.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/pages/settings.scss](diffs/scss/externals__gitlab__stylesheets__pages__settings.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
