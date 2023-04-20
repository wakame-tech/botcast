import { Scene } from "../model.ts";

export const rawSplitLines = (text: string, speaker: string): Scene[] => {
  const scenes: Scene[] = [];
  let buf = "";
  for (const c of text.split("")) {
    if (c === "\n") {
      continue;
    }
    buf += c;
    if (c === "。" || c === "」") {
      scenes.push({
        id: `${scenes.length}`,
        type: "serif",
        speaker,
        text: buf.trim(),
      });
      buf = "";
    }
  }
  return scenes;
};
