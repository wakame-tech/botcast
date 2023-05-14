import { z } from "./deps.ts";

export interface Feed {
  id: string;
  title: string;
  description: string;
  date: Date;
  url: string;
  script: string;
}

export type FeedDigest = Omit<Feed, "script">;

export const Serif = z.object({
  type: z.enum(["serif"]),
  id: z.string(),
  speaker: z.string(),
  text: z.string(),
  kana: z.string(),
});

export type Serif = z.infer<typeof Serif>;

export const Scene = z.object({
  id: z.string(),
  name: z.string(),
  url: z.string().nullable(),
  serifs: z.array(Serif),
});

export type Scene = z.infer<typeof Scene>;

export const Script = z.object({
  id: z.string(),
  url: z.string().nullable(),
  title: z.string(),
  scenes: z.array(Scene),
});

export type Script = z.infer<typeof Script>;
