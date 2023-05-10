import { processText } from "../src/script/index.ts";
// import { upload } from "../src/voicevox.ts";

const text = await Deno.readTextFile("data/hamern.html");

const script = processText({
  type: "hamern",
  title: "hamern",
  text,
});

await Deno.writeTextFile("./hamern.json", JSON.stringify(script, null, 2));

// await upload(script);
