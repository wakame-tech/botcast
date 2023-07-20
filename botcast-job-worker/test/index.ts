import { repo } from "../src/repo.ts";

await repo.save({
  id: crypto.randomUUID(),
  status: "queued",
  arg: "https://syosetu.org/novel/265990/",
  date: new Date().toISOString(),
});
