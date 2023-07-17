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
    _seriesUrl: string,
    html: HTMLDocument
  ): string[] {
    const aTags = Array.from(html.querySelectorAll("dd.subtitle > a"));
    return aTags
      .map((a) => attrValue(a as Element, "href"))
      .map((link) => `https://ncode.syosetu.com${link}`);
  },
  seriesTitle: function (html: HTMLDocument): string {
    return html.querySelector("title")?.textContent ?? "";
  },
  episodeTitle: function (html: HTMLDocument): string {
    return html.querySelector("title")?.textContent.split("-")[1] ?? "";
  },
  body: function (html: HTMLDocument): Serif[] {
    return R.pipe(
      html.querySelector("#novel_honbun")!,
      extractPContents,
      R.map((line) => ({ speaker: "", text: line }))
    );
  },
};
