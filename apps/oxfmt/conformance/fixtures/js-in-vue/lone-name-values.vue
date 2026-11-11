<template>
  <div v-if="  visible  " :a="aVeryLongIdentifierNameThatGoesWellBeyondThePrintWidthOfEightyCharactersForSure" @click="onClick" #default="slotProps">{{  message  }}</div>
  <div :t=" true " :f="false" :n="null" :b="  undefined  " @c="  $emit  " v-slot="eval">{{ aVeryLongIdentifierNameThatGoesBeyondThePrintWidthOfEightyCharactersToo }}</div>
  <div :class="class" :d="café" v-for="  item   in   items  " #e="arguments">{{ this }}</div>
  <div :a="someObject.aVeryLongPropertyNameThatGoesWellBeyondThePrintWidthOfEightyChars" :b=" this.x " :c="a.class" :d="$t.café" :e="true.x" :f="a.b.c">{{ item.aVeryLongPropertyNameThatGoesWellBeyondThePrintWidthOfEightyCharacters }}</div>
  <!-- In parens, the same through the parse -->
  <div :a="(someObject.aVeryLongPropertyNameThatGoesWellBeyondThePrintWidthOfEightyChars)" :b="(name)">{{ (item.aVeryLongPropertyNameThatGoesWellBeyondThePrintWidthOfEightyCharacters) }}</div>
</template>
