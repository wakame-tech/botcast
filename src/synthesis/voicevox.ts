const origin = "http://127.0.0.1:50021";

type Query = {
  speedScale: number;
  kana: string;
};

export const queryVoiceVox = async (
  text: string,
  speaker: string
): Promise<Query> => {
  const queryUrl = `${origin}/audio_query?text=${encodeURI(
    text
  )}&speaker=${speaker}`;
  const res = await fetch(queryUrl, {
    method: "POST",
  }).catch((e) => console.log(e));
  if (!res) {
    throw "failed";
  }
  const query: Query = await res.json();
  query["speedScale"] = 1.5;
  return query;
};

export const synthesisVoiceVox = async (
  query: Query,
  speaker: string,
  outPath: string
): Promise<string> => {
  const synthesisUrl = `${origin}/synthesis?speaker=${speaker}`;
  const res = await fetch(synthesisUrl, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(query),
  }).catch((e) => console.log(e));
  if (!res) {
    throw "failed";
  }
  const voice = await res.blob();
  await Deno.writeFile(outPath, voice.stream());
  return outPath;
};
