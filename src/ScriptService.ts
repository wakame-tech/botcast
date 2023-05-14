import { Scene, Script, Serif, Source } from "./model.ts";
import { hamern } from "./repo/script/hamern.ts";
import { concatAudios } from "./synthesis/ffmpeg.ts";
import { queryVoiceVox, synthesisVoiceVox } from "./synthesis/voicevox.ts";
import { SHA256 } from "https://denopkg.com/chiefbiiko/sha256@v1.0.0/mod.ts";
import { raw } from "./repo/script/raw.ts";
import { narou } from "./repo/script/narou.ts";

export const toHash = (texts: string[]): string => {
  const hasher = new SHA256();
  for (const text of texts) {
    hasher.update(text);
  }
  return hasher.digest("hex") as string;
};

export function existsSync(filepath: string): boolean {
  try {
    const file = Deno.statSync(filepath);
    return file.isFile;
  } catch (e) {
    return false;
  }
}

const fetchHtml = async (url: string): Promise<string> => {
  const hash = toHash([url]);
  const cachePath = `data/_sources/${hash}.html`;
  if (existsSync(cachePath)) {
    console.log(`[fetch] using cache ${url}`);
    return Deno.readTextFile(cachePath);
  } else {
    const html = await fetch(url).then((res) => res.text());
    await Deno.writeTextFile(cachePath, html);
    console.log(`[fetch] cached ${url}`);
    return html;
  }
};

export class ScriptService {
  constructor() {}

  async processSource(source: Source): Promise<Scene> {
    const speaker = "31";
    const processor = (source: Source) => {
      if (!source.url) {
        return raw;
      } else if (source.url.startsWith("https://syosetu.org")) {
        return hamern;
      } else if (source.url.startsWith("https://ncode.syosetu.com")) {
        return narou;
      } else {
        return raw;
      }
    };
    const html = await fetchHtml(source.url);
    const lines = processor(source)(html);
    const serifs: Serif[] = [];
    for await (const line of lines) {
      const query = await queryVoiceVox(line, speaker).catch((e) =>
        console.log(e)
      );
      if (!query) {
        throw "query failed";
      }
      const kana = query.kana;
      const serif: Serif = {
        type: "serif",
        id: toHash([speaker, kana]),
        speaker,
        text: line,
        kana,
      };
      serifs.push(serif);
    }
    return {
      id: toHash(serifs.map((serif) => serif.id)),
      url: source.url ?? null,
      name: `scene`,
      serifs,
    };
  }

  async generate(title: string, sources: Source[]): Promise<Script> {
    const scenes: Scene[] = await Promise.all(sources.map(this.processSource));
    const script: Script = {
      id: toHash(scenes.map((scene) => scene.id)),
      title,
      scenes,
    };
    return script;
  }

  async synthesis(script: Script): Promise<ArrayBuffer> {
    const dir = `data/_wavs`;
    const serifs = script.scenes.flatMap((scene) => scene.serifs);
    const outPaths: string[] = [];
    for await (const serif of serifs) {
      const outPath = `${dir}/${serif.id}.wav`;
      const cached = existsSync(outPath);
      if (!cached) {
        const query = await queryVoiceVox(serif.text, serif.speaker);
        await synthesisVoiceVox(query, serif.speaker, outPath);
      }
      console.log(
        `[voicevox] ${cached ? "cached" : "generated"} ${serif.id.slice(0, 6)}`
      );
      console.log(serif.text);
      console.log(serif.kana);
      outPaths.push(outPath);
    }
    const output = `data/_podcasts/${script.id}.mp3`;
    if (!existsSync(output)) {
      await concatAudios(outPaths, output);
    }
    const audio = await Deno.readFile(output);
    console.log(
      `[synthesis] ${script.title} ${script.scenes.length} scenes (${Math.floor(
        audio.byteLength / 1000
      )} KB)`
    );
    return audio;
  }
}

export const scriptService = new ScriptService();
