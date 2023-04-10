import { Feed, Script, Serif } from "./model.ts";
import { PodcastRepository, podcastRepository } from "./supabase.ts";
// import { nanoid } from "https://deno.land/x/nanoid/mod.ts";
import { FeedRepository, feedRepository } from "./FeedRepository.ts";
import { getFeedXML } from "./rss.ts";
import { exec } from "./deps.ts";

const origin = "http://localhost:50021";

export class VoiceVoxFeedGenerator {
  constructor(
    private podcastRepository: PodcastRepository,
    private feedRepository: FeedRepository
  ) {}

  async generateSerif(serif: Serif): Promise<string> {
    let res = await fetch(
      `${origin}/audio_query?text=${serif.text}&speaker=${serif.speaker}`,
      {
        method: "POST",
      }
    );
    if (res.status === 422) {
      throw await res.json();
    }

    const query = await res.json();
    query["speedScale"] = 1.5;
    res = await fetch(`${origin}/synthesis?speaker=${serif.speaker}`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(query),
    });
    if (res.status === 422) {
      throw await res.json();
    }
    const voice = await res.arrayBuffer();
    const path = `voices/${serif.id}.wav`;
    await Deno.writeFile(path, new Uint8Array(voice));
    console.debug(`generated id=${serif.id} ${serif.speaker} > ${serif.text}`);
    return path;
  }

  async generate(script: Script): Promise<Feed> {
    const wavs = await Promise.all(
      script.scenes.map((scene) => this.generateSerif(scene))
    ).catch((e) => console.error(e));
    if (!wavs) {
      throw "";
    }
    const inputs = wavs.map((wav) => `file '${wav}'`).join("\n");
    const output = `${script.id}.wav`;

    await Deno.writeTextFile("input.txt", inputs);
    const cmd = `ffmpeg -y -f concat -i input.txt ${output}`;
    const res = await exec(cmd);
    console.log(res);
    await Deno.remove("input.txt");
    await Promise.all(wavs.map((wav) => Deno.remove(wav)));

    const data = await Deno.readFile(output);

    const url = await this.podcastRepository.upload(script.id, data.buffer);
    await Deno.remove(output);
    const feed = {
      id: script.id,
      title: script.title,
      description: "desc",
      date: new Date(),
      url,
    };
    await this.feedRepository.create(feed).catch((e) => console.log(e));
    return feed;
  }
}

const splitLines = (text: string): string[] => {
  const lines = [];
  let buf = "";
  for (const c of text.split("")) {
    if (c === "\n") {
      continue;
    }
    buf += c;
    if (c === "。" || c === "」") {
      lines.push(buf.trim());
      buf = "";
    }
  }
  return lines;
};

export const upload = async (path: string, title: string): Promise<void> => {
  const text = await Deno.readTextFile(path);
  const lines = splitLines(text);
  const script = {
    id: crypto.randomUUID(),
    title,
    scenes: lines.map((line, index) => ({
      type: "serif",
      id: `${index}`,
      speaker: "31",
      text: line,
    })),
  } satisfies Script;
  await generator.generate(script);
};

const generator = new VoiceVoxFeedGenerator(podcastRepository, feedRepository);
await upload("maou001.txt", "魔王学院の不適合者 1");
await upload("maou002.txt", "魔王学院の不適合者 2");
const xml = await getFeedXML(feedRepository);
console.log(xml);
