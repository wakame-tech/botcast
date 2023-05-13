import { exec } from "../deps.ts";

export const concatAudios = async (
  paths: string[],
  outPath: string
): Promise<void> => {
  const inputTxtPath = `data/_wavs/input.txt`;
  const inputs = paths
    .map((path) => `file '${path.split("/").at(-1)}'`)
    .join("\n");
  await Deno.writeTextFile(inputTxtPath, inputs);
  const cmd = `ffmpeg -y -f concat -i ${inputTxtPath} ${outPath}`;
  console.log(`[ffmpeg] ${cmd}`);
  await exec(cmd);

  await Deno.remove(inputTxtPath);
  // await Promise.all(wavs.map((wav) => Deno.remove(wav)));
};
