import { appRouter } from "./router.ts";
import { Hono } from "hono";
import { cors } from "hono/cors";
import { trpcServer } from "@hono/trpc-server";
import { serveStatic } from "hono/deno";
import { createClient } from "jsr:@supabase/supabase-js@2";

const app = new Hono();

export const supabase = createClient(
    Deno.env.get("SUPABASE_URL")!,
    Deno.env.get("SUPABASE_ANON_KEY")!,
);

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
app.use("/*", serveStatic({ root: "./dist/" }));

Deno.serve({ port: 1234 }, app.fetch);
