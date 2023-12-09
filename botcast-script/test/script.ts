import { narou, hamern } from "../src/extractor/index.ts";
import { scrapeSeries } from "../src/scrape.ts";

const series = await scrapeSeries(
  "https://ncode.syosetu.com/n0904cx/",
  // "https://syosetu.org/novel/265990/",
  // "https://ncode.syosetu.com/n1578dx/",
  narou
);

await Deno.writeTextFile(
  `sources/${series.title}.json`,
  JSON.stringify(series, null, 2)
);
