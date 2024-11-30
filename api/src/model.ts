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
  z.object({
    type: z.literal("newEpisode"),
    podcastId: z.string(),
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

const sectionSchema = z.discriminatedUnion("type", [serifSchema]);

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

export type Comment = WithSerializedDates<CommentPrisma>;

export type Script = Omit<ScriptPrisma, "template"> & {
  template: Record<string, unknown>;
};

export const weekDays = z.enum([
  "Mon",
  "Tue",
  "Wed",
  "Thu",
  "Fri",
  "Sat",
  "Sun",
]);
