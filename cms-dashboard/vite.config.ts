import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      '/collections': 'http://localhost:3002',
      '/records': 'http://localhost:3002',
      '/jobs': 'http://localhost:3002',
      '/scripts': 'http://localhost:3002',
      '/canvas': 'http://localhost:3002',
    },
  },
})
