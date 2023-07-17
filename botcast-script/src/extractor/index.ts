import {
  Element,
  HTMLDocument,
} from "https://deno.land/x/deno_dom@v0.1.38/deno-dom-wasm.ts";
import { Episode, Series, Serif } from "../model.ts";

export const attrValue = (a: Element, attrName: string): string => {
  return (
    Array.from(a.attributes).filter((attr) => attr.nodeName === attrName)[0]
      .value ?? ""
  );
};

export const newEpisode = (
  extractor: Extractor,
  url: string,
  html: HTMLDocument
): Promise<Episode> => {
  return Promise.resolve({
    title: extractor.episodeTitle(html),
    url,
    serifs: extractor.body(html),
  });
};

export const newSeries = (
  extractor: Extractor,
  url: string,
  html: HTMLDocument,
  episodes: Episode[]
): Promise<Series> => {
  return Promise.resolve({
    title: extractor.episodeTitle(html),
    url,
    episodes,
  });
};

export interface Extractor {
  collectEpisodeUrls(seriesUrl: string, html: HTMLDocument): string[];
  seriesTitle(html: HTMLDocument): string;
  episodeTitle(html: HTMLDocument): string;
  body(html: HTMLDocument): Serif[];
}

export { extractor as narou } from "./narou.ts";
export { extractor as hamern } from "./hamern.ts";
