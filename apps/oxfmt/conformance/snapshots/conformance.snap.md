## js-in-vue

### Option 1: 422/424 (99.53%)

```json
{"printWidth":80}
```

| File | Note |
| :--- | :--- |
| [externals/prettier/vue/multiparser/lang-tsx.vue](diffs/js-in-vue/externals__prettier__vue__multiparser__lang-tsx.vue.md) | `lang=tsx` is not supported |
| [externals/vue-vben-admin/effects/common-ui/src/components/api-component/api-component.vue](diffs/js-in-vue/externals__vue-vben-admin__effects__common-ui__src__components__api-component__api-component.vue.md) | `<T = any,>() => {}` comma in generic param is removed even in .ts(x) file |

### Option 2: 422/424 (99.53%)

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

### Option 1: 18/19 (94.74%)

```json
{"printWidth":80}
```

| File | Note |
| :--- | :--- |
| [externals/prettier/js/multiparser-css/styled-components.js](diffs/css-in-js/externals__prettier__js__multiparser-css__styled-components.js.md) | `Xxx.extend` not recognized as tag |

### Option 2: 18/19 (94.74%)

```json
{"printWidth":100}
```

| File | Note |
| :--- | :--- |
| [externals/prettier/js/multiparser-css/styled-components.js](diffs/css-in-js/externals__prettier__js__multiparser-css__styled-components.js.md) | `Xxx.extend` not recognized as tag |

## html-in-js

### Option 1: 188/191 (98.43%)

```json
{"printWidth":80}
```

| File | Note |
| :--- | :--- |
| [externals/prettier/js/multiparser-html/issue-10691.js](diffs/html-in-js/externals__prettier__js__multiparser-html__issue-10691.js.md) | js-in-html(`<script>`)-in-js needs lot more work; Please see oxc_formatter/src/print/template/embed/html.rs |
| [externals/webawesome/number-input/number-input.styles.ts](diffs/html-in-js/externals__webawesome__number-input__number-input.styles.ts.md) | Layout-only: Prettier's fill fit-check breaks inside `var()` args in a long `calc()`; ours breaks after the operator. See crates/oxc_formatter_css/AGENTS.md |
| [externals/webawesome/page/page.styles.ts](diffs/html-in-js/externals__webawesome__page__page.styles.ts.md) | Layout-only: Prettier's fill fit-check breaks inside `::slotted()` after a long `:not(...)`; ours breaks inside `:not(...)`. See crates/oxc_formatter_css/AGENTS.md |

### Option 2: 190/191 (99.48%)

```json
{"printWidth":100,"htmlWhitespaceSensitivity":"ignore"}
```

| File | Note |
| :--- | :--- |
| [externals/prettier/js/multiparser-html/issue-10691.js](diffs/html-in-js/externals__prettier__js__multiparser-html__issue-10691.js.md) | js-in-html(`<script>`)-in-js needs lot more work; Please see oxc_formatter/src/print/template/embed/html.rs |

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

### Option 1: 5/5 (100.00%)

```json
{"printWidth":80}
```

### Option 2: 5/5 (100.00%)

```json
{"printWith":100}
```

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

### Option 1: 395/409 (96.58%)

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
| [externals/ng-zorro-antd/components/style/themes/default.less](diffs/less/externals__ng-zorro-antd__components__style__themes__default.less.md) | Allowed (layout-only): nested Less math — Prettier's fill fit-check breaks inside the wide chunk, ours breaks the separator (biome fill). See crates/oxc_formatter_css/AGENTS.md |
| [externals/ng-zorro-antd/components/style/themes/variable.less](diffs/less/externals__ng-zorro-antd__components__style__themes__variable.less.md) | Allowed (layout-only): nested Less math — Prettier's fill fit-check breaks inside the wide chunk, ours breaks the separator (biome fill). See crates/oxc_formatter_css/AGENTS.md |
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
| [externals/ng-zorro-antd/components/style/themes/default.less](diffs/less/externals__ng-zorro-antd__components__style__themes__default.less.md) | Allowed (layout-only): nested Less math — Prettier's fill fit-check breaks inside the wide chunk, ours breaks the separator (biome fill). See crates/oxc_formatter_css/AGENTS.md |
| [externals/ng-zorro-antd/components/style/themes/variable.less](diffs/less/externals__ng-zorro-antd__components__style__themes__variable.less.md) | Allowed (layout-only): nested Less math — Prettier's fill fit-check breaks inside the wide chunk, ours breaks the separator (biome fill). See crates/oxc_formatter_css/AGENTS.md |
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

### Option 1: 203/217 (93.55%)

```json
{"printWidth":80}
```

| File | Note |
| :--- | :--- |
| [externals/gitlab/stylesheets/components/content_editor.scss](diffs/scss/externals__gitlab__stylesheets__components__content_editor.scss.md) |  |
| [externals/gitlab/stylesheets/framework/diffs.scss](diffs/scss/externals__gitlab__stylesheets__framework__diffs.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/framework/sidebar.scss](diffs/scss/externals__gitlab__stylesheets__framework__sidebar.scss.md) | Allowed (layout-only): wrapped :not() selector-arg indent (prettier/prettier#16165)<br>logn-expr line-break position |
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
| [externals/gitlab/stylesheets/pages/settings.scss](diffs/scss/externals__gitlab__stylesheets__pages__settings.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |

### Option 2: 205/217 (94.47%)

```json
{"printWidth":100}
```

| File | Note |
| :--- | :--- |
| [externals/gitlab/stylesheets/framework/diffs.scss](diffs/scss/externals__gitlab__stylesheets__framework__diffs.scss.md) | Allowed: media-query operator spacing; Prettier can't space arithmetic ops (prettier/prettier#1811) |
| [externals/gitlab/stylesheets/framework/sidebar.scss](diffs/scss/externals__gitlab__stylesheets__framework__sidebar.scss.md) | Allowed (layout-only): wrapped :not() selector-arg indent (prettier/prettier#16165)<br>logn-expr line-break position |
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
