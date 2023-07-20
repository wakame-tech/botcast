import {
  Element,
  HTMLDocument,
} from "https://deno.land/x/deno_dom@v0.1.38/deno-dom-wasm.ts";
import { Serif } from "../model.ts";

export const attrValue = (a: Element, attrName: string): string => {
  return (
    Array.from(a.attributes).filter((attr) => attr.nodeName === attrName)[0]
      .value ?? ""
  );
};

export interface Page {
  url: string;
  html: HTMLDocument;
}

export interface Extractor {
  collectEpisodeUrls(series: Page): string[];
  seriesTitle(series: Page): string;
  episodeTitle(episode: Page): string;
  body(episode: Page): Serif[];
}

export { extractor as narou } from "./narou.ts";
export { extractor as hamern } from "./hamern.ts";
