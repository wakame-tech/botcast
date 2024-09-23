/* prettier-ignore-start */

/* eslint-disable */

// @ts-nocheck

// noinspection JSUnusedGlobalSymbols

// This file is auto-generated by TanStack Router

import { createFileRoute } from '@tanstack/react-router'

// Import Routes

import { Route as rootRoute } from './routes/__root'

// Create Virtual Routes

const TasksLazyImport = createFileRoute('/tasks')()
const SigninLazyImport = createFileRoute('/signin')()
const IndexLazyImport = createFileRoute('/')()
const EpisodesIndexLazyImport = createFileRoute('/episodes/')()
const UsersUserIdLazyImport = createFileRoute('/users/$userId')()
const EpisodesEpisodeIdLazyImport = createFileRoute('/episodes/$episodeId')()

// Create/Update Routes

const TasksLazyRoute = TasksLazyImport.update({
  path: '/tasks',
  getParentRoute: () => rootRoute,
} as any).lazy(() => import('./routes/tasks.lazy').then((d) => d.Route))

const SigninLazyRoute = SigninLazyImport.update({
  path: '/signin',
  getParentRoute: () => rootRoute,
} as any).lazy(() => import('./routes/signin.lazy').then((d) => d.Route))

const IndexLazyRoute = IndexLazyImport.update({
  path: '/',
  getParentRoute: () => rootRoute,
} as any).lazy(() => import('./routes/index.lazy').then((d) => d.Route))

const EpisodesIndexLazyRoute = EpisodesIndexLazyImport.update({
  path: '/episodes/',
  getParentRoute: () => rootRoute,
} as any).lazy(() =>
  import('./routes/episodes/index.lazy').then((d) => d.Route),
)

const UsersUserIdLazyRoute = UsersUserIdLazyImport.update({
  path: '/users/$userId',
  getParentRoute: () => rootRoute,
} as any).lazy(() => import('./routes/users/$userId.lazy').then((d) => d.Route))

const EpisodesEpisodeIdLazyRoute = EpisodesEpisodeIdLazyImport.update({
  path: '/episodes/$episodeId',
  getParentRoute: () => rootRoute,
} as any).lazy(() =>
  import('./routes/episodes/$episodeId.lazy').then((d) => d.Route),
)

// Populate the FileRoutesByPath interface

declare module '@tanstack/react-router' {
  interface FileRoutesByPath {
    '/': {
      id: '/'
      path: '/'
      fullPath: '/'
      preLoaderRoute: typeof IndexLazyImport
      parentRoute: typeof rootRoute
    }
    '/signin': {
      id: '/signin'
      path: '/signin'
      fullPath: '/signin'
      preLoaderRoute: typeof SigninLazyImport
      parentRoute: typeof rootRoute
    }
    '/tasks': {
      id: '/tasks'
      path: '/tasks'
      fullPath: '/tasks'
      preLoaderRoute: typeof TasksLazyImport
      parentRoute: typeof rootRoute
    }
    '/episodes/$episodeId': {
      id: '/episodes/$episodeId'
      path: '/episodes/$episodeId'
      fullPath: '/episodes/$episodeId'
      preLoaderRoute: typeof EpisodesEpisodeIdLazyImport
      parentRoute: typeof rootRoute
    }
    '/users/$userId': {
      id: '/users/$userId'
      path: '/users/$userId'
      fullPath: '/users/$userId'
      preLoaderRoute: typeof UsersUserIdLazyImport
      parentRoute: typeof rootRoute
    }
    '/episodes/': {
      id: '/episodes/'
      path: '/episodes'
      fullPath: '/episodes'
      preLoaderRoute: typeof EpisodesIndexLazyImport
      parentRoute: typeof rootRoute
    }
  }
}

// Create and export the route tree

export const routeTree = rootRoute.addChildren({
  IndexLazyRoute,
  SigninLazyRoute,
  TasksLazyRoute,
  EpisodesEpisodeIdLazyRoute,
  UsersUserIdLazyRoute,
  EpisodesIndexLazyRoute,
})

/* prettier-ignore-end */

/* ROUTE_MANIFEST_START
{
  "routes": {
    "__root__": {
      "filePath": "__root.tsx",
      "children": [
        "/",
        "/signin",
        "/tasks",
        "/episodes/$episodeId",
        "/users/$userId",
        "/episodes/"
      ]
    },
    "/": {
      "filePath": "index.lazy.tsx"
    },
    "/signin": {
      "filePath": "signin.lazy.tsx"
    },
    "/tasks": {
      "filePath": "tasks.lazy.tsx"
    },
    "/episodes/$episodeId": {
      "filePath": "episodes/$episodeId.lazy.tsx"
    },
    "/users/$userId": {
      "filePath": "users/$userId.lazy.tsx"
    },
    "/episodes/": {
      "filePath": "episodes/index.lazy.tsx"
    }
  }
}
ROUTE_MANIFEST_END */