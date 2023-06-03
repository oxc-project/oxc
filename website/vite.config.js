import { resolve } from 'path'
import { defineConfig } from 'vite'

export default defineConfig({
  server: {
    fs: {
      allow: [__dirname, "../npm/wasm-web"],
    },
  },
  build: {
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
        playground: resolve(__dirname, 'playground/index.html'),
      },
    },
  },
})
