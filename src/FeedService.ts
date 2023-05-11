import { Feed, Script } from "./model.ts";
import { IPodcastRepository, LocalPodcastRepository } from "./supabase.ts";
import { IFeedRepository, MockFeedRepository } from "./FeedRepository.ts";
import { ScriptRepository } from "./ScriptRepository.ts";
import { raw } from "./script/raw.ts";
import { hamern } from "./script/hamern.ts";
import { GenerateScript } from "./index.ts";
import { withSpeaker } from "./script/index.ts";

export class FeedService {
  constructor(
    private podcastRepository: IPodcastRepository,
    private feedRepository: IFeedRepository,
    private scriptRepository: ScriptRepository
  ) {}

  generateScript(req: GenerateScript): Script {
    const lines: string[] = {
      raw,
      hamern,
    }[req.type](req.text);
    const scenes = withSpeaker(lines, "31");
    return {
      id: crypto.randomUUID(),
      title: req.title,
      scenes,
    };
  }

  async postPodcast(script: Script): Promise<Feed> {
    const audio = await this.scriptRepository.generate(script);
    const url = await this.podcastRepository.upload(script.id, audio.buffer);
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

export const service = new FeedService(
  new LocalPodcastRepository(),
  new MockFeedRepository(),
  new ScriptRepository()
);
