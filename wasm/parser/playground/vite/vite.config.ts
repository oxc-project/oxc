import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vite'
import wasm from 'vite-plugin-wasm'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [vue(), wasm()],

  server: {
    fs: {
      // Allow serving files from one root level the project
      allow: ['../..'],
    },
  }
})
