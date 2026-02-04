<!-- Case: __isEmbeddedTypescriptGenericParameters -->
<script setup lang="ts" generic="T extends Record<string  ,   unknown>, U">
import B from "./b.vue";
import { defineComponent } from "vue";
import { Route } from "router";

// Case: __isVueBindings (defineProps/defineEmits)
const { foo,
  bar } = defineProps<{ foo: string;    bar: number }>();
const emit = defineEmits<{ change:
  [value: T] }>();

// oxfmt-ignore
const x =                 1;
</script>

<template>
  <ul>
    <!-- Case: __isVueForBindingLeft (v-for) -->
    <li v-for="item in    items" :key="item.id">
      {{ item.name }}
    </li>
    <li v-for="(item,    index) in items" :key="index">
      {{ index }}: {{ item.name }}
    </li>
    <li v-for="{ id, name     } in items" :key="id">
      {{ name }}
    </li>
    <li v-for="(value, key,     index) in object" :key="key">
      {{ index }}: {{ key }} = {{ value }}
    </li>
    <!-- Case: __isInHtmlAttribute (string literals in v-for) -->
    <li v-for="{ name = 'default'   } in items" :key="name">
      {{ name }}
    </li>

    <!-- Case: __isVueBindings (v-slot) -->
    <MyComponent v-slot="{ item,      index }">
      {{ item }}
    </MyComponent>
    <MyComponent>
      <template #default="{       data }">
        {{ data }}
      </template>
      <template #header="props    ">
        {{ props.title }}
      </template>
    </MyComponent>
    <!-- Case: __isInHtmlAttribute (string literals in v-slot) -->
    <MyComponent v-slot="{ label =    'test' }">
      {{ label }}
    </MyComponent>

    <!-- These should be OK (expression parsers) -->
    <div v-if="condition&&anotherCondition">v-if</div>
    <div v-show="isVisible   ">v-show</div>
    <div :class="{ active:          isActive }">v-bind</div>
    <button @click="handleClick($event          )">v-on</button>
  </ul>
</template>
