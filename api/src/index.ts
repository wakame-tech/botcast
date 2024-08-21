import { opine } from "opine";
import { initTRPC } from "@trpc/server";
import * as trpcExpress from "@trpc/server/adapters/express";
import { PrismaClient } from "@prisma/client";
// import { z } from "zod";

const prisma = new PrismaClient();

const t = initTRPC.create();
const appRouter = t.router({
    users: t.procedure.query(async () => {
        const users = await prisma.user.findMany();
        return { users };
    }),
});
export type AppRouter = typeof appRouter;

const app = opine();
app.use(
    "/trpc",
    // @ts-ignore
    trpcExpress.createExpressMiddleware({ router: appRouter }),
);
app.listen(1234);
