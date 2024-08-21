import { initTRPC } from "@trpc/server";
import { PrismaClient } from "@prisma/client";

export const prisma = new PrismaClient();

const t = initTRPC.create();
export const appRouter = t.router({
    users: t.procedure.query(async () => {
        const users = await prisma.user.findMany();
        return { users };
    }),
});
export type AppRouter = typeof appRouter;
