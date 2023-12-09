import { repo } from "./repo.ts";
import { poll } from "./worker.ts";

import {
  Extractor,
  hamern,
  narou,
} from "../../botcast-script/src/extractor/index.ts";
import { scrapeSeries } from "../../botcast-script/src/scrape.ts";

import { voicevox } from "../../botcast-voicebox/src/voicevox.ts";
import { synthesizeSeries } from "../../botcast-voicebox/src/index.ts";

const selectExtractor = (url: string): Extractor => {
  if (url.startsWith("https://ncode.syosetu.com")) {
    return narou;
  } else if (url.startsWith("https://syosetu.org")) {
    return hamern;
  } else {
    throw "unknown url";
  }
};

poll(repo, async (task) => {
  if (task.type === "generate") {
    const extractor = selectExtractor(task.url);
    const series = await scrapeSeries(task.url, extractor);
    await synthesizeSeries(voicevox, series);
  }
});
