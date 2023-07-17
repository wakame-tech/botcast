import {
  DOMParser,
  HTMLDocument,
} from "https://deno.land/x/deno_dom@v0.1.38/deno-dom-wasm.ts";
import { sha256 } from "https://denopkg.com/chiefbiiko/sha256@v1.0.0/mod.ts";
import { Series } from "./model.ts";
import R from "npm:remeda";
import { andThen, collectPromises } from "./fpUtils.ts";
import { Extractor, newEpisode, newSeries } from "./extractor/index.ts";

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

const parseHtml = (content: string): Promise<HTMLDocument> =>
  Promise.resolve(new DOMParser().parseFromString(content, "text/html")!);

const inspect =
  <T>(templateFn: (t: T) => string): ((t: T) => T) =>
  (t) => {
    console.log(templateFn(t));
    return t;
  };

const scrape = (url: string): Promise<HTMLDocument> =>
  R.pipe(url, fetchCached, andThen(parseHtml));

export const scrapeSeries = (
  seriesUrl: string,
  extractor: Extractor
): Promise<Series> =>
  R.pipe(
    seriesUrl,
    scrape,
    andThen((episodeHtml) =>
      R.pipe(
        extractor.collectEpisodeUrls(seriesUrl, episodeHtml),
        inspect((urls) => `${urls.length} episodes found`),
        // (urls) => urls.slice(0, 3),
        R.map(
          (episodeUrl) => () =>
            R.pipe(
              scrape(episodeUrl),
              andThen((episodeHtml) =>
                newEpisode(extractor, episodeUrl, episodeHtml)
              )
            )
        ),
        collectPromises,
        andThen((episodes) =>
          newSeries(extractor, seriesUrl, episodeHtml, episodes)
        )
      )
    )
  );
