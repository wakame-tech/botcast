import { SupabaseClient } from "../../deps.ts";
import { IPodcastRepository } from "./index.ts";

export class SupabasePodcastRepository implements IPodcastRepository {
  constructor(private supabase: SupabaseClient) {}

  async upload(key: string, data: ArrayBuffer): Promise<string> {
    const expireSecs = 60 * 60 * 24;
    const res = await this.supabase.storage.from("podcasts").upload(key, data, {
      upsert: true,
      contentType: "audio/mp3",
    });
    if (res.error) {
      console.log(res.error);
      throw res.error;
    }
    const res2 = await this.supabase.storage
      .from("podcasts")
      .createSignedUrl(key, expireSecs);
    if (res2.error) {
      throw res2.error;
    }
    return res2.signedURL!;
  }
}
