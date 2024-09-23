import { initTRPC, TRPCError } from "@trpc/server";
import { PrismaClient, User } from "@prisma/client";
import { z } from "zod";
// @ts-ignore
import { createClient } from "jsr:@supabase/supabase-js@2";

const supabase = createClient(
    Deno.env.get("SUPABASE_URL")!,
    Deno.env.get("SUPABASE_ANON_KEY")!,
);

export const prisma = new PrismaClient();

interface Args {
    episode_id: string;
    url: string;
}

interface Context {
    accessToken: string | null;
    user: User;
}

const t = initTRPC.context<Context>().create();

const authProcedure = t.procedure.use(
    async ({ ctx: { accessToken }, next }) => {
        if (!accessToken) {
            throw new Error("Unauthorized");
        }
        const { data: { user: authUser } } = await supabase.auth.getUser(
            accessToken,
        );
        if (!authUser) {
            throw new Error("Invalid access token");
        }
        const user = await prisma.user.findUnique({
            where: {
                auth_id: authUser.id,
            },
        });
        if (!user) {
            throw unauthorized;
        }
        return next({
            ctx: {
                user,
            },
        });
    },
);

const unauthorized = new TRPCError({
    code: "UNAUTHORIZED",
    message: "Unauthorized",
});

export const appRouter = t.router({
    signIn: t.procedure.input(z.object({
        email: z.string(),
        password: z.string(),
    })).query(async ({ input }) => {
        const res = await supabase.auth.signInWithPassword({
            email: input.email,
            password: input.password,
        });
        if (res.error) {
            throw res.error;
        }
        const { data: { session } } = res;
        return session.access_token;
    }),
    me: authProcedure.query(({ ctx: { user } }) => {
        return { user };
    }),
    tasks: authProcedure.query(async ({ ctx: { user } }) => {
        const tasks = await prisma.task.findMany({
            where: {
                user,
            },
        });
        return { tasks };
    }),
    task: authProcedure.input(z.object({
        id: z.string(),
    })).query(async ({ input: { id } }) => {
        const task = await prisma.task.findUnique({
            where: { id },
        });
        if (!task) {
            throw new Error("Task not found");
        }
        return { task };
    }),
    episodes: authProcedure.query(async ({ ctx: { user } }) => {
        const episodes = await prisma.episode.findMany({
            where: {
                user,
            },
        });
        return { episodes };
    }),
    episode: authProcedure.input(z.object({
        id: z.string(),
    })).query(async ({ input: { id } }) => {
        const episode = await prisma.episode.findUnique({
            where: { id },
            include: {
                user: true,
            },
        });
        if (!episode) {
            throw new Error("Episode not found");
        }
        return { episode };
    }),
    newEpisode: authProcedure.input(z.object({
        title: z.string(),
        url: z.string(),
    })).mutation(async ({ ctx: { user }, input: { title, url } }) => {
        const episode = await prisma.episode.create({
            data: { title, user_id: user.id },
        });
        await prisma.task.create({
            data: {
                user_id: user.id,
                status: "PENDING",
                args: {
                    episode_id: episode.id,
                    url,
                } satisfies Args,
            },
        });
        return { episode };
    }),
    updateEpisode: authProcedure.input(z.object({
        id: z.string(),
        title: z.string(),
    })).mutation(async ({ input: { id, title } }) => {
        const episode = await prisma.episode.update({
            where: { id },
            data: { title },
        });
        return { episode };
    }),
    deleteEpisode: authProcedure.input(z.object({
        id: z.string(),
    })).mutation(async ({ input: { id } }) => {
        await prisma.episode.delete({ where: { id } });
        return;
    }),
});
export type AppRouter = typeof appRouter;
