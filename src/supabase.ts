import { SupabaseClient, createClient } from "./deps.ts";

const projectUrl = Deno.env.get("SUPABASE_PROJECT_URL")!;
const apiKey = Deno.env.get("SUPABASE_API_KEY")!;
console.log(apiKey);
export const supabase = createClient(projectUrl, apiKey);

export class PodcastRepository {
  constructor(private supabase: SupabaseClient) {}

  async upload(key: string, data: ArrayBuffer): Promise<string> {
    const res = await this.supabase.storage.from("podcasts").upload(key, data, {
      upsert: true,
      contentType: "audio.wav",
    });
    if (res.error) {
      throw res.error;
    }
    const res2 = this.supabase.storage.from("podcasts").getPublicUrl(key);
    if (res2.error) {
      throw res2.error;
    }
    return res2.publicURL!;
  }
}

export const podcastRepository = new PodcastRepository(supabase);
