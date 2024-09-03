import { initTRPC } from "@trpc/server";
import { PrismaClient } from "@prisma/client";
import { z } from "zod";

export const prisma = new PrismaClient();

const WORKER_URL = Deno.env.get("WORKER_URL");

interface ScrapeTaskInput {
    type: "Scrape";
    episode_id: string;
    url: string;
}

interface Context {
    userId: string | null;
}

const t = initTRPC.context<Context>().create();

export const appRouter = t.router({
    testGetUserId: t.procedure.query(({ ctx }) => {
        return { userId: ctx.userId };
    }),
    tasks: t.procedure.query(async () => {
        const tasks = await prisma.task.findMany();
        return { tasks };
    }),
    task: t.procedure.input(z.object({
        id: z.string(),
    })).query(async ({ input: { id } }) => {
        const task = await prisma.task.findUnique({ where: { id } });
        if (!task) {
            throw new Error("Task not found");
        }
        return { task };
    }),
    episodes: t.procedure.query(async () => {
        const episodes = await prisma.episode.findMany();
        return { episodes };
    }),
    episode: t.procedure.input(z.object({
        id: z.string(),
    })).query(async ({ input: { id } }) => {
        const episode = await prisma.episode.findUnique({ where: { id } });
        if (!episode) {
            throw new Error("Episode not found");
        }
        return { episode };
    }),
    newEpisode: t.procedure.input(z.object({
        title: z.string(),
        url: z.string(),
    })).mutation(async ({ input: { title, url } }) => {
        const episode = await prisma.episode.create({
            data: { title },
        });
        const res = await fetch(`${WORKER_URL}/tasks`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify(
                {
                    type: "Scrape",
                    episode_id: episode.id,
                    url,
                } satisfies ScrapeTaskInput,
            ),
        });
        if (!res.ok) {
            throw new Error("Failed to create task");
        }
        return { episode };
    }),
    updateEpisode: t.procedure.input(z.object({
        id: z.string(),
        title: z.string(),
    })).mutation(async ({ input: { id, title } }) => {
        const episode = await prisma.episode.update({
            where: { id },
            data: { title },
        });
        return { episode };
    }),
    deleteEpisode: t.procedure.input(z.object({
        id: z.string(),
    })).mutation(async ({ input: { id } }) => {
        await prisma.episode.delete({ where: { id } });
        return;
    }),
});
export type AppRouter = typeof appRouter;
