import { appRouter } from "@/src/router.ts";
import { Hono } from "hono";
import { cors } from "hono/cors";
import { trpcServer } from "@hono/trpc-server";
import { serveStatic } from "hono/deno";

const app = new Hono();

app.use(cors());
app.use(
    "/trpc/*",
    // @ts-ignore
    trpcServer({ router: appRouter }),
);
app.use("/*", serveStatic({ root: "./dist/" }));

Deno.serve({ port: 1234 }, app.fetch);
