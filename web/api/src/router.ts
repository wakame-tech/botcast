import { initTRPC } from "@trpc/server";
import { PrismaClient } from "@prisma/client";

export const prisma = new PrismaClient();

const t = initTRPC.create();
export const appRouter = t.router({
    episodes: t.procedure.query(async () => {
        const episodes = await prisma.episode.findMany();
        return { episodes };
    }),
});
export type AppRouter = typeof appRouter;
