import { repo } from "./repo.ts";
import { poll } from "./worker.ts";

import { hamern } from "../../botcast-script/src/extractor/index.ts";
import { scrapeSeries } from "../../botcast-script/src/scrape.ts";

import { voicevox } from "../../botcast-voicebox/src/voicevox.ts";
import { synthesizeSeries } from "../../botcast-voicebox/src/index.ts";

const CACHE_DIR = "./cache";
poll(repo, async (url) => {
  const series = await scrapeSeries(url, hamern);
  await synthesizeSeries(voicevox, series, `${CACHE_DIR}/out.mp3`);
});
