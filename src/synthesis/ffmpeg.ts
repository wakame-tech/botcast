import { wavsDir } from "../config.ts";
import { exec } from "../deps.ts";

export const concatAudios = async (
  paths: string[],
  outPath: string
): Promise<void> => {
  const inputTxtPath = `${wavsDir}/input.txt`;
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
};
