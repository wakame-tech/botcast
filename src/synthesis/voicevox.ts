import { Status } from "https://deno.land/x/oak@v12.1.0/deps.ts";

const origin = "http://127.0.0.1:50021";

export const synthesisVoiceVox = async (
  text: string,
  speaker: string,
  outPath: string
): Promise<string> => {
  const queryUrl = `${origin}/audio_query?text=${encodeURI(
    text
  )}&speaker=${speaker}`;
  let res = await fetch(queryUrl, {
    method: "POST",
  });
  if (res.status === Status.UnprocessableEntity) {
    throw await res.json();
  }

  const query = await res.json();
  query["speedScale"] = 1.5;
  const synthesisUrl = `${origin}/synthesis?speaker=${speaker}`;
  res = await fetch(synthesisUrl, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(query),
  });
  if (res.status === Status.UnprocessableEntity) {
    throw await res.json();
  }
  const voice = await res.arrayBuffer();
  // const path = `voices/${serif.id}.wav`;
  await Deno.writeFile(outPath, new Uint8Array(voice));
  return outPath;
};
