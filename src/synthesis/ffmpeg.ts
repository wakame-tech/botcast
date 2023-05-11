import { exec } from "../deps.ts";

export const concatWavs = async (
  wavs: string[],
  outPath: string
): Promise<void> => {
  const inputs = wavs.map((wav) => `file '${wav}'`).join("\n");
  await Deno.writeTextFile("input.txt", inputs);
  const cmd = `ffmpeg -y -f concat -i input.txt ${outPath}`;
  const res = await exec(cmd);
  console.log(res);
  // await Deno.remove("input.txt");
  await Promise.all(wavs.map((wav) => Deno.remove(wav)));
};
