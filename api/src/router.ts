import { initTRPC, TRPCError } from "@trpc/server";
import { PrismaClient, User } from "prisma_client";
import type {
  Corner,
  Mail,
  Script,
  Sections,
  Task,
  TaskArgs,
} from "./model.ts";
import { createSecret, deleteSecret, listSecrets } from "./vault.ts";
import { z } from "zod";
// @ts-ignore: cannot resolve deps from npm package
import { createClient } from "supabase";
// @ts-ignore: cannot resolve deps from npm package
import { s3 } from "./presign.ts";
import {
  parseEpisode,
  PodcastInputSchema,
  ScriptInputSchema,
  sectionsSchema,
  taskArgsSchema,
  withoutDates,
} from "./model.ts";
// @ts-ignore: cannot resolve deps from npm package
import { isValidSchema, validate } from "npm:jtd";

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

const authProcedure = t.procedure
  .use(async (opts) => {
    const start = Date.now();

    const result = await opts.next();

    const durationMs = Date.now() - start;
    const meta = { path: opts.path, type: opts.type, durationMs };

    result.ok ? console.log("Ok", meta) : console.error("Error", meta, result);

    return result;
  })
  .use(
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
        throw unauthorized();
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
    // @ts-ignore: PrismaClient with extension
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
        createSecret(
          // @ts-ignore: PrismaClient with extension
          prisma,
          user.id,
          secret.value,
          secret.name,
        )
      ),
    );
    await Promise.all(
      input.deletionIds.map((id) =>
        deleteSecret(
          // @ts-ignore: PrismaClient with extension
          prisma,
          user.id,
          id,
        )
      ),
    );
  }),
  tasks: authProcedure.input(z.object({
    includesCompleted: z.boolean().default(false),
  })).query(async ({ ctx: { user }, input: { includesCompleted } }) => {
    const tasks = await prisma.task.findMany({
      where: {
        user,
        status: includesCompleted ? undefined : "PENDING",
      },
      orderBy: {
        executed_at: "desc",
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
  addEvaluateTemplateTaskById: authProcedure.input(z.object({
    id: z.string(),
    cron: z.string(),
  })).mutation(async ({ ctx: { user }, input: { id, cron } }) => {
    const script = await prisma.script.findUnique({
      where: { id },
    });
    if (!script) {
      throw not_found(id);
    }
    if (Object.keys(JSON.parse(JSON.stringify(script.arguments))).length > 0) {
      throw new TRPCError({
        code: "BAD_REQUEST",
        message: "evaluateTemplate task must not have arguments",
      });
    }
    await prisma.task.create({
      data: {
        user_id: user.id,
        cron,
        status: "PENDING",
        args: {
          type: "evaluateTemplate",
          template: JSON.parse(JSON.stringify(script.template)),
          parameters: {},
        } satisfies TaskArgs,
      },
    });
    return;
  }),
  deleteTask: authProcedure.input(z.object({
    id: z.string(),
  })).mutation(async ({ input: { id } }) => {
    await prisma.task.delete({ where: { id } });
    return;
  }),
  topPodcasts: t.procedure.query(async () => {
    const podcasts = await prisma.podcast.findMany({
      include: {
        user: true,
      },
      orderBy: {
        created_at: "desc",
      },
      take: 10,
    });
    return { podcasts: withoutDates(podcasts) };
  }),
  podcasts: authProcedure.query(async ({ ctx: { user } }) => {
    const podcasts = await prisma.podcast.findMany({
      where: {
        user,
      },
      include: {
        user: true,
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
        corners: true,
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
        corners: podcast.corners as Corner[],
      },
    };
  }),
  newPodcast: authProcedure.input(PodcastInputSchema).mutation(
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
  updatePodcast: authProcedure.input(
    z.object({
      id: z.string(),
    }).merge(PodcastInputSchema),
  ).mutation(
    async ({ input: { id, title, icon, description } }) => {
      const podcast = await prisma.podcast.update({
        where: { id },
        data: {
          title,
          icon,
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
    return {
      episode: parseEpisode(episode),
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
    sections: sectionsSchema.optional(),
  })).mutation(async ({ input: { id, title, description, sections } }) => {
    const episode = await prisma.episode.update({
      where: { id },
      data: { title, description, sections },
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
  newScript: authProcedure.input(ScriptInputSchema).mutation(
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
  updateScript: authProcedure.input(ScriptInputSchema.extend({
    id: z.string(),
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
  newCorner: authProcedure.input(z.object({
    podcastId: z.string(),
    title: z.string(),
    description: z.string(),
    mail_schema: z.string().nullable(),
  })).mutation(
    async (
      { ctx: { user }, input: { podcastId, title, description, mail_schema } },
    ) => {
      const podcast = await prisma.podcast.findUnique({
        where: { id: podcastId },
      });
      if (!podcast) {
        throw not_found(podcastId);
      }

      if (mail_schema && !isValidSchema(JSON.parse(mail_schema))) {
        throw new TRPCError({
          code: "BAD_REQUEST",
          message: "Invalid mail schema",
        });
      }

      const corner = await prisma.corner.create({
        data: {
          podcast_id: podcastId,
          title,
          description,
          user_id: user.id,
          requesting_mail: !!mail_schema,
          mail_schema: mail_schema ? JSON.parse(mail_schema) : null,
        },
      });
      return { corner };
    },
  ),
  corner: authProcedure.input(z.object({
    id: z.string(),
  })).query(async ({ input: { id } }) => {
    const corner = await prisma.corner.findUnique({
      where: { id },
    });
    if (!corner) {
      throw not_found(id);
    }
    return {
      corner: corner as Corner,
    };
  }),
  updateCorner: authProcedure.input(z.object({
    id: z.string(),
    title: z.string(),
    description: z.string(),
    mail_schema: z.string().nullable(),
  })).mutation(
    async ({ input: { id, title, description, mail_schema } }) => {
      await prisma.corner.update({
        where: { id },
        data: {
          title,
          description,
          requesting_mail: !mail_schema,
          mail_schema: mail_schema ? JSON.parse(mail_schema) : null,
        },
      });
      return;
    },
  ),
  deleteCorner: authProcedure.input(z.object({
    id: z.string(),
  })).mutation(async ({ input: { id } }) => {
    await prisma.corner.delete({ where: { id } });
    return;
  }),
  newMail: authProcedure.input(z.object({
    cornerId: z.string(),
    body: z.record(z.unknown()),
  })).mutation(
    async ({ ctx: { user }, input: { cornerId, body } }) => {
      const corner = await prisma.corner.findUnique({
        where: { id: cornerId },
      });
      if (!corner) {
        throw not_found(cornerId);
      }
      if (!corner.mail_schema) {
        throw new TRPCError({
          code: "BAD_REQUEST",
          message: "This corner does not accept mail",
        });
      }
      const errors = validate(
        corner.mail_schema as object,
        body,
      );
      if (errors.length > 0) {
        throw new TRPCError({
          code: "BAD_REQUEST",
          message: JSON.stringify(errors),
        });
      }
      const mail = await prisma.mail.create({
        data: {
          body: JSON.parse(JSON.stringify(body)),
          corner_id: cornerId,
          user_id: user.id,
        },
      });
      return { mail };
    },
  ),
  mails: authProcedure.input(z.object({
    cornerId: z.string(),
  })).query(async ({ input: { cornerId } }) => {
    const mails = await prisma.mail.findMany({
      where: {
        corner_id: cornerId,
      },
    });
    return { mails: withoutDates(mails as Mail[]) };
  }),
  deleteMail: authProcedure.input(z.object({
    id: z.string(),
  })).mutation(async ({ input: { id } }) => {
    await prisma.mail.delete({ where: { id } });
    return;
  }),
});
export type AppRouter = typeof appRouter;
