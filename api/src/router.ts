import { initTRPC, TRPCError } from "@trpc/server";
import { PrismaClient, User } from "@prisma/client";
import type { Script, Sections, Task } from "./model.ts";
import { createSecret, deleteSecret, listSecrets } from "./vault.ts";
import { z } from "zod";
// @ts-ignore: cannot resolve deps from npm package
import { createClient } from "jsr:@supabase/supabase-js@2";
// @ts-ignore: cannot resolve deps from npm package
import { s3 } from "./presign.ts";
import {
  parseEpisode,
  sectionsSchema,
  taskArgsSchema,
  withoutDates,
} from "./model.ts";

const supabase = createClient(
  Deno.env.get("SUPABASE_URL")!,
  Deno.env.get("SUPABASE_ANON_KEY")!,
);

export const prisma = new PrismaClient();

interface Context {
  accessToken: string | null;
  user: User;
}

const t = initTRPC.context<Context>().create();

const authProcedure = t.procedure.use(
  async ({ ctx: { accessToken }, next }) => {
    if (!accessToken) {
      throw unauthorized();
    }
    if (accessToken === Deno.env.get("SUPABASE_SERVICE_ROLE_KEY")) {
      const user = await prisma.user.findUnique({
        where: {
          auth_id: Deno.env.get("SUPABASE_SERVICE_ROLE_USER_ID"),
        },
      });
      if (!user) {
        throw unauthorized();
      }
      return next({
        ctx: {
          user,
        },
      });
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

const unauthorized = (e?: unknown) =>
  new TRPCError({
    code: "UNAUTHORIZED",
    message: "Unauthorized",
    cause: e,
  });

const not_found = (id: string) =>
  new TRPCError({
    code: "NOT_FOUND",
    message: `${id} Not found`,
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
      throw unauthorized(res.error);
    }
    const { data: { session } } = res;
    return session.access_token;
  }),
  me: authProcedure.query(({ ctx: { user } }) => {
    return { user };
  }),
  secrets: authProcedure.query(async ({ ctx: { user } }) => {
    const secrets = await listSecrets(prisma, user.id);
    return {
      secrets: secrets.map((s) => ({
        id: s.id,
        name: s.name.split(":", 2)[1],
      })),
    };
  }),
  updateSecrets: authProcedure.input(z.object({
    news: z.array(z.object({
      name: z.string(),
      value: z.string(),
    })),
    deletionIds: z.array(z.string()),
  })).mutation(async ({ ctx: { user }, input }) => {
    await Promise.all(
      input.news.map((secret) =>
        createSecret(prisma, user.id, secret.value, secret.name)
      ),
    );
    await Promise.all(
      input.deletionIds.map((id) => deleteSecret(prisma, user.id, id)),
    );
  }),
  tasks: authProcedure.query(async ({ ctx: { user } }) => {
    const tasks = await prisma.task.findMany({
      where: {
        user,
      },
    });
    return { tasks: withoutDates(tasks as Task[]) };
  }),
  task: authProcedure.input(z.object({
    id: z.string(),
  })).query(async ({ input: { id } }) => {
    const task = await prisma.task.findUnique({
      where: { id },
    });
    if (!task) {
      throw not_found(id);
    }
    return {
      task: task as Task,
    };
  }),
  addTask: authProcedure.input(z.object({
    cron: z.string().nullable(),
    args: taskArgsSchema,
  })).mutation(
    async ({ ctx: { user }, input: { args, cron } }) => {
      const task = await prisma.task.create({
        data: {
          user_id: user.id,
          cron,
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
    return { podcasts: withoutDates(podcasts) };
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
      throw not_found(id);
    }
    return {
      podcast: {
        ...podcast,
        episodes: withoutDates(podcast.episodes),
        user: podcast.user,
      },
    };
  }),
  newPodcast: authProcedure.input(z.object({
    title: z.string(),
    description: z.string().optional(),
    icon: z.string().regex(/\p{Emoji_Presentation}/gu),
  })).mutation(
    async (
      { ctx: { user }, input: { title, description, icon } },
    ) => {
      const podcast = await prisma.podcast.create({
        data: {
          title,
          description,
          icon,
          user_id: user.id,
          created_at: new Date().toISOString(),
        },
      });
      return { podcast };
    },
  ),
  updatePodcast: authProcedure.input(z.object({
    id: z.string(),
    title: z.string(),
    description: z.string().optional(),
  })).mutation(
    async ({ input: { id, title, description } }) => {
      const podcast = await prisma.podcast.update({
        where: { id },
        data: {
          title,
          description,
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
      throw not_found(id);
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
    const clientEpisode = parseEpisode(episode);
    const comments = withoutDates(episode.comments);
    return {
      episode: {
        ...clientEpisode,
        comments,
      },
    };
  }),
  newEpisode: authProcedure.input(z.object({
    podcastId: z.string(),
    title: z.string(),
    description: z.string().optional(),
    sections: sectionsSchema,
  })).mutation(
    async (
      { ctx: { user }, input: { podcastId, title, description, sections } },
    ) => {
      const episode = await prisma.episode.create({
        data: {
          title,
          description,
          user_id: user.id,
          podcast_id: podcastId,
          sections,
          created_at: new Date().toISOString(),
        },
      });
      return { episode: parseEpisode(episode) };
    },
  ),
  updateEpisode: authProcedure.input(z.object({
    id: z.string(),
    title: z.string(),
    description: z.string().optional(),
  })).mutation(async ({ input: { id, title, description } }) => {
    const episode = await prisma.episode.update({
      where: { id },
      data: { title, description },
    });
    return {
      ...episode,
      sections: episode.sections as Sections,
    };
  }),
  scripts: authProcedure.query(async ({ ctx: { user } }) => {
    const scripts = await prisma.script.findMany({
      where: {
        user,
      },
    });
    return { scripts: scripts as Script[] };
  }),
  script: authProcedure.input(z.object({
    id: z.string(),
  })).query(async ({ input: { id } }) => {
    const script = await prisma.script.findUnique({
      where: { id },
    });
    if (!script) {
      throw not_found(id);
    }
    return { script: script as Script };
  }),
  newScript: authProcedure.input(z.object({
    title: z.string(),
    description: z.string().optional(),
    template: z.string(),
  })).mutation(
    async ({ ctx: { user }, input: { title, description, template } }) => {
      const templateJson = JSON.parse(template);
      const script = await prisma.script.create({
        data: {
          title,
          description,
          template: templateJson,
          user_id: user.id,
        },
      });
      return { script };
    },
  ),
  updateScript: authProcedure.input(z.object({
    id: z.string(),
    title: z.string(),
    description: z.string().optional(),
    template: z.string(),
  })).mutation(async ({ input: { id, title, description, template } }) => {
    const templateJson = JSON.parse(template);
    await prisma.script.update({
      where: { id },
      data: {
        title,
        description,
        template: templateJson,
      },
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
