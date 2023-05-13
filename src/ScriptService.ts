import { Scene, Script, Serif } from "./model.ts";
import { Source } from "./routes.ts";
import { hamern } from "./repo/script/hamern.ts";
import { concatAudios } from "./synthesis/ffmpeg.ts";
import { synthesisVoiceVox } from "./synthesis/voicevox.ts";
import { SHA256 } from "https://denopkg.com/chiefbiiko/sha256@v1.0.0/mod.ts";
import { raw } from "./repo/script/raw.ts";
import { narou } from "./repo/script/narou.ts";

const toHash = (texts: string[]): string => {
  const hasher = new SHA256();
  for (const text of texts) {
    hasher.update(text);
  }
  return hasher.digest("hex") as string;
};

function existsSync(filepath: string): boolean {
  try {
    const file = Deno.statSync(filepath);
    return file.isFile;
  } catch (e) {
    return false;
  }
}

export class ScriptService {
  constructor() {}

  generate(title: string, url: string | null, sources: Source[]): Script {
    const processor = (source: Source) => {
      if (source.url === null) {
        return raw;
      } else if (source.url.startsWith("https://syosetu.org")) {
        return hamern;
      } else if (source.url.startsWith("https://ncode.syosetu.com")) {
        return narou;
      } else {
        return raw;
      }
    };

    const scenes = sources.map((source) => {
      const serifs = processor(source)(source.text).map((text) => {
        const speaker = "31";
        return {
          type: "serif",
          id: toHash([speaker, text]),
          speaker,
          text,
        } satisfies Serif;
      });

      return {
        id: toHash(serifs.map((serif) => serif.id)),
        url: source.url,
        name: source.title,
        serifs,
      } satisfies Scene;
    });
    const script: Script = {
      id: toHash(scenes.map((scene) => scene.id)),
      title,
      url,
      scenes,
    };
    return script;
  }

  async synthesis(script: Script): Promise<ArrayBuffer> {
    const dir = `data/_wavs`;
    const serifs = script.scenes.flatMap((scene) => scene.serifs);
    const outPaths = await Promise.all(
      serifs.map(async (serif) => {
        const outPath = `${dir}/${serif.id}.wav`;
        if (!existsSync(outPath)) {
          await synthesisVoiceVox(serif.text, serif.speaker, outPath);
          console.log(`[voicevox] ${serif.text} \n${outPath}`);
        }
        return outPath;
      })
    );
    const output = `data/_podcasts/${script.id}.mp3`;
    await concatAudios(outPaths, output);
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
