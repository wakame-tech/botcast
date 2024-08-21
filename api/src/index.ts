import { opine } from "opine";
import { initTRPC } from "@trpc/server";
import * as trpcExpress from "@trpc/server/adapters/express";
// import { z } from "zod";

const t = initTRPC.create();
const appRouter = t.router({
    ping: t.procedure.query(() => "pong"),
});
export type AppRouter = typeof appRouter;

const app = opine();
app.use(
    "/trpc",
    // @ts-ignore
    trpcExpress.createExpressMiddleware({ router: appRouter }),
);
app.listen(1234);
