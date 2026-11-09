<template>
  <CustomComponent>
    <template v-for="day in days" #[`slot.${day.split(',')[0]}`]>
      x
    </template>
  </CustomComponent>
  <div :[key.split('-')[0]]="value" @[`on${name.split(',')[0]}`]="handler"></div>
</template>
