import { IPodcastRepository } from "./index.ts";

export class LocalPodcastRepository implements IPodcastRepository {
  async upload(key: string, data: ArrayBuffer): Promise<string> {
    const path = `data/${key}.mp3`;
    await Deno.writeFile(path, new Uint8Array(data));
    return path;
  }

  async delete(key: string): Promise<void> {
    const path = `data/${key}.mp3`;
    await Deno.remove(path);
  }
}
