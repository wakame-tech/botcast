import { SupabaseClient } from "../../deps.ts";
import { IPodcastRepository } from "./index.ts";

export class SupabasePodcastRepository implements IPodcastRepository {
  constructor(private supabase: SupabaseClient) {}

  async upload(key: string, audio: ArrayBuffer): Promise<string> {
    const expireSecs = 60 * 60 * 24;
    const { error: uploadErr } = await this.supabase.storage
      .from("podcasts")
      .upload(key, audio, {
        upsert: true,
        contentType: "audio/mp3",
      });
    if (uploadErr) {
      throw uploadErr;
    }
    const { data, error: createUrlErr } = await this.supabase.storage
      .from("podcasts")
      .createSignedUrl(key, expireSecs);
    if (createUrlErr) {
      throw createUrlErr;
    }
    return data!.signedURL!;
  }

  async delete(key: string): Promise<void> {
    const { error } = await this.supabase.storage
      .from("podcasts")
      .remove([key]);
    if (error) {
      throw error;
    }
    return;
  }
}
