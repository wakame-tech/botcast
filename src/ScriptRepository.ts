import { Script } from "./model.ts";
import { concatWavs } from "./synthesis/ffmpeg.ts";
import { synthesisVoiceVox } from "./synthesis/voicevox.ts";

export class ScriptRepository {
  constructor() {}

  async generate(script: Script): Promise<Uint8Array> {
    const wavs = await Promise.all(
      script.scenes.map((serif) => {
        const wavPath = `voices/${serif.id}.wav`;
        return synthesisVoiceVox(serif.text, serif.speaker, wavPath);
      })
    );
    const output = `${script.id}.wav`;
    await concatWavs(wavs, output);
    return Deno.readFile(output);
  }
}
