import { createClient } from "../deps.ts";

const projectUrl = Deno.env.get("SUPABASE_PROJECT_URL")!;
const apiKey = Deno.env.get("SUPABASE_API_KEY")!;
export const supabase = createClient(projectUrl, apiKey);
