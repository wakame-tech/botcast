import { Episode, Series } from "../../botcast-script/src/model.ts";
import { concatAudios } from "./ffmpeg.ts";
import { sha256 } from "https://denopkg.com/chiefbiiko/sha256@v1.0.0/mod.ts";
import tqdm from "npm:tqdm";

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
  for (const serif of tqdm(episode.serifs)) {
    const outPath = `${CACHE_DIR}/${crypto.randomUUID()}.wav`;
    // console.log(`synthesize ${serif.text} ${outPath}`);
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
  series: Series
): Promise<void> => {
  for (const episode of series.episodes) {
    const paths = await synthesizeEpisode(synthesizer, episode);
    const fileName = sha256(episode.title, "utf-8", "hex");
    const outPath = `${CACHE_DIR}/${fileName}.mp3`;
    await concatAudios(paths, outPath);
  }
};
