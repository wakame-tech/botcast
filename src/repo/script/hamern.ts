import { DOMParser } from "https://deno.land/x/deno_dom@v0.1.38/deno-dom-wasm.ts";
import { extractRuby, replaceRubyToRt } from "./ruby.ts";

export const hamern = (text: string): string[] => {
  const doc = new DOMParser().parseFromString(text, "text/html")!;
  const el = doc.querySelector("#honbun")!;
  const lines = Array.from(el.children).filter((e) => e.nodeName === "P");

  return lines
    .map((e) =>
      replaceRubyToRt(
        cleanText(e.textContent),
        Array.from(e.children)
          .filter((e) => e.tagName === "RUBY")
          .map(extractRuby),
        ["(", ")"]
      ).trim()
    )
    .filter((text) => text !== "");
};

const cleanText = (text: string): string => {
  return text.replaceAll(/※/g, "");
};
