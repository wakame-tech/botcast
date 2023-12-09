import { Series } from "../botcast-script/src/model.ts";
import { synthesizeSeries } from "./src/index.ts";
import { voicevox } from "./src/voicevox.ts";

const series: Series = JSON.parse(
  await Deno.readTextFile(
    "../botcast-script/sources/神童セフィリアの下剋上プログラム.json"
  )
);
await synthesizeSeries(voicevox, series);
