import React from 'react'
import ReactDOM from 'react-dom/client'
import { RouterProvider, createRouter } from '@tanstack/react-router'
import 'virtual:uno.css'

import { routeTree } from './routeTree.gen'
const router = createRouter({ routeTree })

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }
}

ReactDOM.createRoot(
  // biome-ignore lint:
  document.getElementById('root')!
).render(
  <React.StrictMode>
    <RouterProvider router={router} />
  </React.StrictMode>,
)
