import {
  DOMParser,
  HTMLDocument,
} from "https://deno.land/x/deno_dom@v0.1.38/deno-dom-wasm.ts";
import { sha256 } from "https://denopkg.com/chiefbiiko/sha256@v1.0.0/mod.ts";
import { Episode, Series } from "./model.ts";
import R from "npm:remeda";
import { andThen, collectPromises } from "./fpUtils.ts";
import { Extractor, Page } from "./extractor/index.ts";

async function fileExists(filepath: string): Promise<boolean> {
  try {
    const file = await Deno.stat(filepath);
    return file.isFile;
  } catch (e) {
    return false;
  }
}

export const fetchCached = async (
  url: string,
  cooldown = 1000
): Promise<string> => {
  const hash = sha256(url, "utf8", "hex");
  const cachePath = `cache/${hash}.html`;
  const exists = await fileExists(cachePath);
  console.log(`${exists} ${url} (${hash})`);
  if (exists) {
    return Deno.readTextFile(cachePath);
  } else {
    await new Promise((res) => setTimeout(res, cooldown));
    const res = await fetch(url).then((res) => res.text());
    await Deno.writeTextFile(cachePath, res);
    console.log(`cached ${url} (${hash})`);
    return res;
  }
};

const pure = <T>(t: T): Promise<T> => Promise.resolve(t);

const parseHtml = (content: string): Promise<HTMLDocument> =>
  pure(new DOMParser().parseFromString(content, "text/html")!);

const inspect =
  <T>(templateFn: (t: T) => string): ((t: T) => T) =>
  (t) => {
    console.log(templateFn(t));
    return t;
  };

const scrape = (url: string): Promise<Page> =>
  R.pipe(
    url,
    fetchCached,
    andThen(parseHtml),
    andThen((html) => pure({ url, html }))
  );

export const scrapeEpisode = (
  extractor: Extractor,
  episodeUrl: string
): Promise<Episode> =>
  R.pipe(
    scrape(episodeUrl),
    andThen((page) =>
      pure({
        title: extractor.episodeTitle(page),
        url: episodeUrl,
        serifs: extractor.body(page),
      } satisfies Episode)
    )
  );

export const scrapeSeries = (
  seriesUrl: string,
  extractor: Extractor
): Promise<Series> =>
  R.pipe(
    scrape(seriesUrl),
    andThen((page) =>
      R.pipe(
        extractor.collectEpisodeUrls(page),
        inspect((urls) => `${urls.length} episodes found`),
        (urls) => urls.slice(0, 1),
        R.map((episodeUrl) => () => scrapeEpisode(extractor, episodeUrl)),
        collectPromises,
        andThen((episodes) =>
          pure({
            title: extractor.seriesTitle(page),
            url: seriesUrl,
            episodes,
          } satisfies Series)
        )
      )
    )
  );
