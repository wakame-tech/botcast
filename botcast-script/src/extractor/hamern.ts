import R from "npm:remeda";
import { Element } from "https://deno.land/x/deno_dom@v0.1.38/deno-dom-wasm.ts";
import { extractPContents } from "../util.ts";
import { Extractor, Page, attrValue } from "./index.ts";
import { Serif } from "../model.ts";

export const extractor: Extractor = {
  collectEpisodeUrls: (page: Page): string[] => {
    const aTags = Array.from(page.html.querySelectorAll(".ss > table a"));
    return aTags
      .map((a) => attrValue(a as Element, "href"))
      .map((link) => `${page.url}${link.replace("./", "")}`);
  },
  seriesTitle: (page: Page): string => {
    return (
      page.html.querySelector("title")?.textContent?.split("-")[0].trim() ?? ""
    );
  },
  episodeTitle: (page: Page): string => {
    return (
      page.html.querySelector("title")?.textContent?.split("-")[0].trim() ?? ""
    );
  },
  body: (page: Page): Serif[] => {
    return R.pipe(
      page.html.querySelector("#honbun")!,
      extractPContents,
      R.map((line) => ({ speaker: "", text: line }))
    );
  },
};
