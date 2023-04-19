import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import WindiCSS from 'vite-plugin-windicss'

// https://vitejs.dev/config/
export default defineConfig({
  optimizeDeps: {
    exclude: ["codemirror", "@codemirror/language-javascript" /* ... */],
  },
  plugins: [
    svelte(),
    WindiCSS(),
  ],
  server: {
    fs: {
      allow: [process.cwd(), "../npm/wasm-web"],
    },
  },
})
