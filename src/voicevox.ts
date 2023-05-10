import { Feed, Script, Serif } from "./model.ts";
import { IPodcastRepository, LocalPodcastRepository } from "./supabase.ts";
import { IFeedRepository, MockFeedRepository } from "./FeedRepository.ts";
import { exec } from "./deps.ts";
import { Status } from "https://deno.land/x/oak@v12.1.0/deps.ts";

const origin = "http://127.0.0.1:50021";

export class VoiceVoxFeedGenerator {
  constructor(
    private podcastRepository: IPodcastRepository,
    private feedRepository: IFeedRepository
  ) {}

  async generateSerif(serif: Serif): Promise<string> {
    console.log(serif.text);
    const queryUrl = `${origin}/audio_query?text=${encodeURI(
      serif.text
    )}&speaker=${serif.speaker}`;
    let res = await fetch(queryUrl, {
      method: "POST",
    });
    if (res.status === Status.UnprocessableEntity) {
      throw await res.json();
    }

    const query = await res.json();
    query["speedScale"] = 1.5;
    const synthesisUrl = `${origin}/synthesis?speaker=${serif.speaker}`;
    res = await fetch(synthesisUrl, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(query),
    });
    if (res.status === Status.UnprocessableEntity) {
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
    );
    const inputs = wavs.map((wav) => `file '${wav}'`).join("\n");
    const output = `${script.id}.wav`;

    await Deno.writeTextFile("input.txt", inputs);
    const cmd = `ffmpeg -y -f concat -i input.txt ${output}`;
    const res = await exec(cmd);
    console.log(res);
    // await Deno.remove("input.txt");
    await Promise.all(wavs.map((wav) => Deno.remove(wav)));

    const data = await Deno.readFile(output);

    const url = await this.podcastRepository.upload(script.id, data.buffer);
    // await Deno.remove(output);
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

export const upload = async (script: Script): Promise<Feed> => {
  const generator = new VoiceVoxFeedGenerator(
    new LocalPodcastRepository(),
    new MockFeedRepository()
  );
  const feed = await generator.generate(script);
  return feed;
};
