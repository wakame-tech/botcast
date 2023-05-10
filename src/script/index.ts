import { z } from "../deps.ts";
import { Scene, Script } from "../model.ts";
import { hamern } from "./hamern.ts";
import { raw } from "./raw.ts";

export const GenerateScript = z.object({
  title: z.string(),
  type: z.enum(["raw", "hamern"]),
  text: z.string(),
});

export type GenerateScript = z.infer<typeof GenerateScript>;

const withSpeaker = (lines: string[], speakerId: string): Scene[] => {
  return lines.map((line) => {
    return {
      type: "serif",
      id: crypto.randomUUID(),
      speaker: speakerId,
      text: line,
    };
  });
};

export const processText = (req: GenerateScript): Script => {
  const lines: string[] = {
    raw,
    hamern,
  }[req.type](req.text);
  const scenes = withSpeaker(lines, "31");
  return {
    id: crypto.randomUUID(),
    title: req.title,
    scenes,
  };
};
