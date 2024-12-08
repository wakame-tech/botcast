import { z } from "zod";
import {
  Comment as CommentPrisma,
  Episode as EpisodePrisma,
  Podcast as PodcastPrisma,
  Script as ScriptPrisma,
  Task as TaskPrisma,
} from "@prisma/client";

// Workaround for https://github.com/prisma/prisma/discussions/5522
export type WithSerializedDates<Type> = {
  [Key in keyof Type]: Type[Key] extends Date ? string
    : Type[Key] extends Date | null ? string | null
    : Type[Key] extends Date | undefined ? string | undefined
    : Type[Key] extends Date | null | undefined ? string | null | undefined
    : Type[Key];
};

export const withoutDates = <T>(model: T): WithSerializedDates<T> =>
  JSON.parse(JSON.stringify(model));

export const taskArgsSchema = z.discriminatedUnion("type", [
  z.object({
    type: z.literal("generateAudio"),
    episodeId: z.string(),
  }),
  z.object({
    type: z.literal("evaluateTemplate"),
    template: z.any(),
    context: z.record(z.any()),
  }),
]);

export type TaskArgs = z.infer<typeof taskArgsSchema>;

export type Task = Omit<TaskPrisma, "args" | "result"> & {
  args: TaskArgs;
  result: Record<string, unknown> | null;
};

const serifSchema = z.object({
  type: z.literal("Serif"),
  speaker: z.string(),
  text: z.string(),
});

const audioSchema = z.object({
  type: z.literal("Audio"),
  url: z.string(),
  from: z.number().optional(),
  to: z.number().optional(),
});

const sectionSchema = z.discriminatedUnion("type", [serifSchema, audioSchema]);

export const sectionsSchema = z.array(sectionSchema);

export type Sections = z.infer<typeof sectionsSchema>;

export type Episode = Omit<EpisodePrisma, "sections"> & {
  sections: Sections;
};

export const parseEpisode = (
  episode: EpisodePrisma,
): WithSerializedDates<Episode> =>
  withoutDates({
    ...episode,
    sections: sectionsSchema.parse(episode.sections),
  });

export type Podcast = WithSerializedDates<PodcastPrisma>;

export const PodcastInputSchema = z.object({
  icon: z.string().regex(/\p{Emoji_Presentation}/gu),
  title: z.string(),
  description: z.string().nullable(),
});

export type PodcastInput = z.infer<typeof PodcastInputSchema>;

export type Comment = WithSerializedDates<CommentPrisma>;

export type Script = Omit<ScriptPrisma, "template"> & {
  template: Record<string, unknown>;
};

export const ScriptInputSchema = z.object({
  title: z.string(),
  description: z.string().nullable(),
  template: z.string(),
});

export type ScriptInput = z.infer<typeof ScriptInputSchema>;
