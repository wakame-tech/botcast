import R from "npm:remeda";
import {
  Element,
  HTMLDocument,
} from "https://deno.land/x/deno_dom@v0.1.38/deno-dom-wasm.ts";
import { extractPContents } from "../util.ts";
import { Extractor, attrValue } from "./index.ts";
import { Serif } from "../model.ts";

export const extractor: Extractor = {
  collectEpisodeUrls: function (
    seriesUrl: string,
    html: HTMLDocument
  ): string[] {
    const aTags = Array.from(html.querySelectorAll(".ss > table a"));
    return aTags
      .map((a) => attrValue(a as Element, "href"))
      .map((link) => `${seriesUrl}${link.replace("./", "")}`);
  },
  seriesTitle: function (html: HTMLDocument): string {
    return html.querySelector("title")?.textContent?.split("-")[0].trim() ?? "";
  },
  episodeTitle: function (html: HTMLDocument): string {
    return html.querySelector("title")?.textContent?.split("-")[0].trim() ?? "";
  },
  body: function (html: HTMLDocument): Serif[] {
    return R.pipe(
      html.querySelector("#honbun")!,
      extractPContents,
      R.map((line) => ({ speaker: "", text: line }))
    );
  },
};
