import { Series } from "../botcast-script/src/model.ts";
import { synthesizeSeries } from "./src/index.ts";
import { voicevox } from "./src/voicevox.ts";

const series: Series = JSON.parse(
  await Deno.readTextFile("./scripts/青い星は流れたか.json")
);
await synthesizeSeries(voicevox, series, "scripts/out.mp3");
