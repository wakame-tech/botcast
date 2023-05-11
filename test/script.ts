import { service } from "../src/FeedService.ts";
import { ScriptRepository } from "../src/ScriptRepository.ts";
// import { upload } from "../src/voicevox.ts";

const text = await Deno.readTextFile("data/hamern.html");
const script = service.generateScript({
  type: "hamern",
  title: "hamern",
  text,
});
const generator = new ScriptRepository();
await generator.generate(script);

await Deno.writeTextFile("./hamern.json", JSON.stringify(script, null, 2));

// await upload(script);
