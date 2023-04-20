import { z } from "../deps.ts";
import { Scene, Script } from "../model.ts";
import { rawSplitLines } from "./raw.ts";

export const GenerateScript = z.object({
  title: z.string(),
  type: z.enum(["raw", "narou"]),
  text: z.string(),
});

export type GenerateScript = z.infer<typeof GenerateScript>;

export interface TextProcessor {
  process(text: string): Scene[];
}

export const processText = (req: GenerateScript): Script => {
  const scenes = {
    raw: (text: string) => rawSplitLines(text, "31"),
    narou: (text: string) => {
      throw "TODO";
    },
  }[req.type](req.text);
  return {
    id: crypto.randomUUID(),
    title: req.title,
    scenes,
  };
};
