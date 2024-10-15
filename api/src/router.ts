import { initTRPC, TRPCError } from "@trpc/server";
import { PrismaClient, User } from "@prisma/client";
import { z } from "zod";
// @ts-ignore: cannot resolve deps from npm package
import { createClient } from "jsr:@supabase/supabase-js@2";

const supabase = createClient(
    Deno.env.get("SUPABASE_URL")!,
    Deno.env.get("SUPABASE_ANON_KEY")!,
);

export const prisma = new PrismaClient();

const taskArgsSchema = z.union([
    z.object({
        type: z.literal("generateAudio"),
        episodeId: z.string(),
    }),
    z.object({
        type: z.literal("evaluateScript"),
        episodeId: z.string(),
    }),
]);

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

type Section = {
    type: "Serif";
    speaker: string;
    text: string;
};

interface Manuscript {
    title: string;
    sections: Section[];
}

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
            throw new TRPCError({
                code: "NOT_FOUND",
                message: "Task not found",
            });
        }
        return { task };
    }),
    addTask: authProcedure.input(taskArgsSchema).mutation(
        async ({ ctx: { user }, input: args }) => {
            const task = await prisma.task.create({
                data: {
                    user_id: user.id,
                    status: "PENDING",
                    args,
                },
            });
            return { task };
        },
    ),
    podcasts: authProcedure.query(async ({ ctx: { user } }) => {
        const podcasts = await prisma.podcast.findMany({
            where: {
                user,
            },
        });
        return { podcasts };
    }),
    podcast: authProcedure.input(z.object({
        id: z.string(),
    })).query(async ({ input: { id } }) => {
        const podcast = await prisma.podcast.findUnique({
            where: { id },
            include: {
                user: true,
                episodes: true,
            },
        });
        if (!podcast) {
            throw new Error("Episode not found");
        }
        return { podcast };
    }),
    newPodcast: authProcedure.input(z.object({
        title: z.string(),
        icon: z.string().regex(/\p{Emoji_Presentation}/gu),
    })).mutation(async ({ ctx: { user }, input: { title, icon } }) => {
        const template = {
            title: "title",
            sections: [
                {
                    type: "Serif",
                    speaker: "urn:voicevox:zunda_normal",
                    text: "こんにちは",
                },
            ],
        } satisfies Manuscript;
        const script = await prisma.script.create({
            data: {
                title: `${title} script`,
                template,
                user_id: user.id,
            },
        });
        const podcast = await prisma.podcast.create({
            data: {
                title,
                icon,
                script_id: script.id,
                user_id: user.id,
                created_at: new Date().toISOString(),
            },
        });
        return { podcast };
    }),
    deletePodcast: authProcedure.input(z.object({
        id: z.string(),
    })).mutation(async ({ input: { id } }) => {
        await prisma.podcast.delete({ where: { id } });
        return;
    }),
    episode: authProcedure.input(z.object({
        id: z.string(),
    })).query(async ({ input: { id } }) => {
        const episode = await prisma.episode.findUnique({
            where: { id },
            include: {
                user: true,
                script: true,
                comments: {
                    include: {
                        user: true,
                    },
                },
            },
        });
        if (!episode) {
            throw new Error("Episode not found");
        }
        return { episode };
    }),
    newEpisode: authProcedure.input(z.object({
        podcastId: z.string(),
    })).mutation(
        async ({ ctx: { user }, input: { podcastId } }) => {
            const podcast = await prisma.podcast.findUnique({
                where: { id: podcastId },
            });
            if (!podcast) {
                throw new Error("Podcast not found");
            }
            const episode = await prisma.episode.create({
                include: {
                    script: true,
                },
                data: {
                    script_id: podcast.script_id,
                    podcast_id: podcastId,
                    title: "new episode",
                    user_id: user.id,
                    created_at: new Date().toISOString(),
                },
            });
            return { episode };
        },
    ),
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
    updateScript: authProcedure.input(z.object({
        id: z.string(),
        template: z.string(),
    })).mutation(async ({ input: { id, template } }) => {
        const templateJson = JSON.parse(template);
        await prisma.script.update({
            where: { id },
            data: { template: templateJson },
        });
        return;
    }),
    deleteEpisode: authProcedure.input(z.object({
        id: z.string(),
    })).mutation(async ({ input: { id } }) => {
        await prisma.episode.delete({ where: { id } });
        return;
    }),
    newComment: authProcedure.input(z.object({
        episodeId: z.string(),
        content: z.string(),
    })).mutation(async ({ ctx: { user }, input: { episodeId, content } }) => {
        await prisma.comment.create({
            data: {
                content,
                user_id: user.id,
                episode_id: episodeId,
                created_at: new Date().toISOString(),
            },
        });
    }),
    deleteComment: authProcedure.input(z.object({
        id: z.string(),
    })).mutation(async ({ input: { id } }) => {
        await prisma.comment.delete({ where: { id } });
    }),
});
export type AppRouter = typeof appRouter;
