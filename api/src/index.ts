import { appRouter } from "./router.ts";
import { Hono } from "hono";
import { cors } from "hono/cors";
import { trpcServer } from "@hono/trpc-server";
import { serveStatic } from "hono/deno";

const app = new Hono();

app.use(cors());
app.use(
    "/trpc/*",
    // @ts-ignore: TS2339
    trpcServer({
        router: appRouter,
        createContext: (opts) => {
            const accessToken = opts.req.headers.get("Authorization");
            return {
                accessToken,
                ...opts,
            };
        },
    }),
);
app
    .use("/assets/*", serveStatic({ root: "./dist/" }))
    .use(
        "/*",
        serveStatic({
            root: "./dist/",
            path: "index.html",
        }),
    );

Deno.serve({ port: 1234 }, app.fetch);
