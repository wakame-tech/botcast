import { feedService } from "../src/FeedService.ts";
import { scriptService } from "../src/ScriptService.ts";
import { Script } from "../src/model.ts";

const upload = async (script: Script): Promise<void> => {
  const audio = await scriptService.synthesis(script);
  const feed = await feedService.post(script, audio);
  console.log(feed);
};

// const script = scriptService.generate(
//   "青い星は流れたか1",
//   "https://syosetu.org/novel/265990",
//   [
//     {
//       title: "第1章『じゃあ暇ね』",
//       url: "https://syosetu.org/novel/265990/1.html",
//       text: await Deno.readTextFile("data/hamern_265990/1.html"),
//     },
//     {
//       title: "第2章『柚之木七夕へ』",
//       url: "https://syosetu.org/novel/265990/2.html",
//       text: await Deno.readTextFile("data/hamern_265990/2.html"),
//     },
//     {
//       title: "第3章『臨時バスはどこへ行く』",
//       url: "https://syosetu.org/novel/265990/3.html",
//       text: await Deno.readTextFile("data/hamern_265990/3.html"),
//     },
//   ]
// );
// await Deno.writeTextFile(
//   `./data/265590/script.json`,
//   JSON.stringify(script, null, 2)
// );
// await upload(script);
// console.log(await feedService.getFeeds());

const script = scriptService.generate(
  "魔王学院の不適合者　～史上最強の魔王の始祖、転生して子孫たちの学校へ通う～",
  "https://ncode.syosetu.com/n1578dx",
  [
    {
      title: "プロローグ　～転生～",
      url: "https://ncode.syosetu.com/n1578dx/1/",
      text: await Deno.readTextFile("data/narou_n1578dx/1.html"),
    },
    {
      title: "デルゾゲードからの招待状",
      url: "https://ncode.syosetu.com/n1578dx/2/",
      text: await Deno.readTextFile("data/narou_n1578dx/2.html"),
    },
  ]
);
await Deno.writeTextFile(
  `./data/narou_n1578dx/script.json`,
  JSON.stringify(script, null, 2)
);
await upload(script);
