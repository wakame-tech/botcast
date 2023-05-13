import { configSync } from "https://deno.land/std@0.167.0/dotenv/mod.ts";

configSync({
  export: true,
  path: "./.env",
});

export {
  SupabaseClient,
  createClient,
} from "https://deno.land/x/supabase@1.3.1/mod.ts";
export { OutputMode, exec } from "https://deno.land/x/exec@0.0.5/mod.ts";
export * from "https://deno.land/x/oak@v12.1.0/mod.ts";
export { z } from "https://deno.land/x/zod/mod.ts";
