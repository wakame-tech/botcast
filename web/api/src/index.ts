import { appRouter } from "@/src/router.ts";
import { Hono } from "hono";
import { cors } from "hono/cors";
import { createClient } from "jsr:@supabase/supabase-js@2";
import { trpcServer } from "@hono/trpc-server";
import { serveStatic } from "hono/deno";

const app = new Hono();

app.use(cors());
app.use(
    "/trpc/*",
    // @ts-ignore
    trpcServer({
        router: appRouter,
        createContext: async (opts) => {
            const supabase = createClient(
                Deno.env.get("SUPABASE_URL")!,
                Deno.env.get("SUPABASE_ANON_KEY")!,
            );

            const authorization = opts.req.headers.get("Authorization");
            if (authorization) {
                const { data: { user } } = await supabase.auth.getUser(
                    authorization,
                );
                return {
                    userId: user?.id,
                    ...opts,
                };
            }
            return opts;
        },
    }),
);
app.use("/*", serveStatic({ root: "./dist/" }));

Deno.serve({ port: 1234 }, app.fetch);
