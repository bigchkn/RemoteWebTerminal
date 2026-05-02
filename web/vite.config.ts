import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  build: {
    outDir: 'dist',
    rollupOptions: {
      output: {
        entryFileNames: 'app.js',
        chunkFileNames: 'chunk-[hash].js',
        assetFileNames: ({ name }) =>
          name?.endsWith('.css') ? 'styles.css' : (name ?? '[name][extname]'),
      },
    },
  },
})
