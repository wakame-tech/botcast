import { z } from "./deps.ts";

export interface Feed {
  id: string;
  title: string;
  description: string;
  date: Date;
  url: string;
}

export type Scene = Serif;
// | Sound;

export const Scene = z.object({
  type: z.enum(["serif"]),
  id: z.string(),
  speaker: z.string(),
  text: z.string(),
});

export interface Serif {
  type: "serif";
  id: string;
  speaker: string;
  text: string;
}

export interface Sound {
  type: "sound";
  id: string;
  path: string;
}

export interface Script {
  id: string;
  title: string;
  scenes: Scene[];
}

export const Script = z.object({
  id: z.string(),
  title: z.string(),
  scenes: z.array(Scene),
});
