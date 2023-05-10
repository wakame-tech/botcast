export const raw = (text: string): string[] => {
  const lines: string[] = [];
  let buf = "";
  for (const c of text.split("")) {
    if (c === "\n") {
      continue;
    }
    buf += c;
    if (c === "。" || c === "」") {
      lines.push(buf.trim());
      buf = "";
    }
  }
  return lines;
};
