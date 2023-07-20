import { exec } from "https://deno.land/x/exec@0.0.5/mod.ts";
import { CACHE_DIR } from "./index.ts";

export const concatAudios = async (
  paths: string[],
  outPath: string
): Promise<void> => {
  const inputTxtPath = `${CACHE_DIR}/input.txt`;
  const inputs = paths
    .map((path) => `file '${path.split("/").at(-1)}'`)
    .join("\n");
  await Deno.writeTextFile(inputTxtPath, inputs);
  const cmd = `ffmpeg -y -f concat -i ${inputTxtPath} ${outPath}`;
  console.log(`[ffmpeg] ${cmd}`);
  const res = await exec(cmd);
  if (!res.status.success) {
    console.log(inputTxtPath);
    throw res;
  }
  await Deno.remove(inputTxtPath);
  for (const path of paths) {
    await Deno.remove(path);
  }
};
