import { Scene } from "../model.ts";

export const withSpeaker = (lines: string[], speakerId: string): Scene[] => {
  return lines.map((line) => {
    return {
      type: "serif",
      id: crypto.randomUUID(),
      speaker: speakerId,
      text: line,
    };
  });
};
