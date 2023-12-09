import { Series } from "../src/model.ts";
import { summerizeSeries } from "../src/transformer/summarizer.ts";

const series: Series = JSON.parse(
  await Deno.readTextFile(`sources/神童セフィリアの下剋上プログラム.json`)
);

const summerizedSeries = await summerizeSeries(series);
await Deno.writeTextFile(
  `sources/神童セフィリアの下剋上プログラム-summary.json`,
  JSON.stringify(summerizedSeries, null, 2)
);
