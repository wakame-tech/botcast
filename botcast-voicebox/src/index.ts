import { Episode, Series } from "../../botcast-script/src/model.ts";
import { concatAudios } from "./ffmpeg.ts";

export const CACHE_DIR = `cache`;

export interface Synthesizer {
  assignSpeaker(speaker: string): string;
  synthesize(speaker: string, text: string, outPath: string): Promise<void>;
}

export const synthesizeEpisode = async (
  synthesizer: Synthesizer,
  episode: Episode
): Promise<string[]> => {
  console.log(`synthesize ${episode.title} ${episode.url}`);
  const res = [];
  for (const serif of episode.serifs) {
    const outPath = `${CACHE_DIR}/${crypto.randomUUID()}.wav`;
    console.log(`synthesize ${serif.text} ${outPath}`);
    await synthesizer.synthesize(
      synthesizer.assignSpeaker(serif.speaker),
      serif.text,
      outPath
    );
    res.push(outPath);
  }
  return res;
};

export const synthesizeSeries = async (
  synthesizer: Synthesizer,
  series: Series,
  outPath: string
): Promise<void> => {
  let paths: string[] = [];
  for (const episode of series.episodes) {
    paths = [...paths, ...(await synthesizeEpisode(synthesizer, episode))];
  }
  await concatAudios(paths, outPath);
};
