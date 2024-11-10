import { initTRPC, TRPCError } from "@trpc/server";
import { PrismaClient, User } from "@prisma/client";
import { z } from "zod";
// @ts-ignore: cannot resolve deps from npm package
import { createClient } from "jsr:@supabase/supabase-js@2";
// @ts-ignore: cannot resolve deps from npm package
import { s3 } from "@/src/presign.ts";

const supabase = createClient(
    Deno.env.get("SUPABASE_URL")!,
    Deno.env.get("SUPABASE_ANON_KEY")!,
);

export const prisma = new PrismaClient();

const taskArgsSchema = z.discriminatedUnion("type", [
    z.object({
        type: z.literal("generateAudio"),
        episodeId: z.string(),
    }),
    z.object({
        type: z.literal("evaluateScript"),
        scriptId: z.string(),
    }),
    z.object({
        type: z.literal("newEpisode"),
        podcastId: z.string(),
    }),
]);

const weekDays = z.enum([
    "Mon",
    "Tue",
    "Wed",
    "Thu",
    "Fri",
    "Sat",
    "Sun",
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
                script: true,
            },
        });
        if (!podcast) {
            throw new Error("Episode not found");
        }
        return { podcast };
    }),
    newPodcast: authProcedure.input(z.object({
        title: z.string(),
        template: z.string(),
        icon: z.string().regex(/\p{Emoji_Presentation}/gu),
        weekDay: weekDays,
        // JST
        hour: z.number().int().min(0).max(23),
    })).mutation(
        async (
            { ctx: { user }, input: { title, template, icon, weekDay, hour } },
        ) => {
            const script = await prisma.script.create({
                data: {
                    title: `${title} script`,
                    template: JSON.parse(template),
                    user_id: user.id,
                },
            });
            const podcast = await prisma.podcast.create({
                data: {
                    title,
                    icon,
                    script_id: script.id,
                    user_id: user.id,
                    cron: `0 0 ${(hour + 9) % 24} * * ${weekDay}`,
                    created_at: new Date().toISOString(),
                },
            });
            return { podcast };
        },
    ),
    updatePodcast: authProcedure.input(z.object({
        id: z.string(),
        title: z.string(),
        weekDay: weekDays,
        hour: z.number().int().min(0).max(23),
    })).mutation(
        async ({ input: { id, title, weekDay, hour } }) => {
            const podcast = await prisma.podcast.update({
                where: { id },
                data: {
                    title,
                    cron: `0 0 ${(hour + 9) % 24} * * ${weekDay}`,
                },
            });
            return { podcast };
        },
    ),
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
        if (episode.audio_url && episode.srt_url) {
            try {
                const audioPresignUrl = await s3.getPresignedUrl(
                    "GET",
                    episode.audio_url,
                );
                episode.audio_url = audioPresignUrl ?? null;
                const srtPresignUrl = await s3.getPresignedUrl(
                    "GET",
                    episode.srt_url,
                );
                episode.srt_url = srtPresignUrl ?? null;
            } catch (e) {
                console.error(e);
                throw new TRPCError({
                    code: "INTERNAL_SERVER_ERROR",
                    message: "Failed to get presigned URL",
                    cause: e,
                });
            }
        }
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
    scripts: authProcedure.query(async ({ ctx: { user } }) => {
        const scripts = await prisma.script.findMany({
            where: {
                user,
            },
        });
        return { scripts };
    }),
    script: authProcedure.input(z.object({
        id: z.string(),
    })).query(async ({ input: { id } }) => {
        const script = await prisma.script.findUnique({
            where: { id },
        });
        if (!script) {
            throw new Error("Script not found");
        }
        return { script };
    }),
    newScript: authProcedure.input(z.object({
        title: z.string(),
        template: z.string(),
    })).mutation(async ({ ctx: { user }, input: { title, template } }) => {
        const templateJson = JSON.parse(template);
        const script = await prisma.script.create({
            data: {
                title,
                template: templateJson,
                user_id: user.id,
            },
        });
        return { script };
    }),
    updateScript: authProcedure.input(z.object({
        id: z.string(),
        title: z.string(),
        template: z.string(),
    })).mutation(async ({ input: { id, title, template } }) => {
        const templateJson = JSON.parse(template);
        await prisma.script.update({
            where: { id },
            data: { title, template: templateJson },
        });
        return;
    }),
    deleteScript: authProcedure.input(z.object({
        id: z.string(),
    })).mutation(async ({ input: { id } }) => {
        await prisma.script.delete({ where: { id } });
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
