import { configSync } from "https://deno.land/std@0.167.0/dotenv/mod.ts";

configSync({
  export: true,
  path: "./.env",
});

export {
  SupabaseClient,
  createClient,
} from "https://deno.land/x/supabase@1.3.1/mod.ts";
export type { Item } from "https://esm.sh/podcast@2.0.1";
export { Podcast } from "https://esm.sh/podcast@2.0.1";
export { OutputMode, exec } from "https://deno.land/x/exec@0.0.5/mod.ts";
