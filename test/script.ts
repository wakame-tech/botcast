import { supabase } from "../src/lib/supabase.ts";
import { Source } from "../src/model.ts";
import { SupabaseGenerateTaskRepository } from "../src/repo/feed/GenerateTaskRepository.ts";
import type { PostTaskRequest } from "../src/routes.ts";

const origin = "http://localhost:8000/api/v1";

const taskRepo = new SupabaseGenerateTaskRepository(supabase);
await taskRepo.clean();

const url = "https://ncode.syosetu.com/n1578dx";
const sources: Source[] = [...Array(5)].map(
  (_, i) =>
    ({
      url: `${url}/${i + 1}.html`,
    } satisfies Source)
);
console.log(`${sources.length} sources`);
for await (const source of sources) {
  const res = await fetch(`${origin}/feeds`, {
    method: "POST",
    body: JSON.stringify({
      title: "test",
      sources: [source],
    } satisfies PostTaskRequest),
  }).then((res) => res.json());
  console.log(res);
}
