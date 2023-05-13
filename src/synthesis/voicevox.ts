export const synthesisVoiceVox = async (
  text: string,
  speaker: string,
  outPath: string
): Promise<string> => {
  const origin = "http://127.0.0.1:50021";
  const queryUrl = `${origin}/audio_query?text=${encodeURI(
    text
  )}&speaker=${speaker}`;
  console.log(`[voicevox] ${queryUrl}`);
  let res = await fetch(queryUrl, {
    method: "POST",
  }).catch((e) => console.log(e));
  if (!res) {
    throw "failed";
  }
  const query = await res.json();
  query["speedScale"] = 1.5;
  const synthesisUrl = `${origin}/synthesis?speaker=${speaker}`;
  console.log(`[voicevox] ${synthesisUrl}`);
  res = await fetch(synthesisUrl, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(query),
  }).catch((e) => console.log(e));
  if (!res) {
    throw "failed";
  }
  const voice = await res.arrayBuffer();
  await Deno.writeFile(outPath, new Uint8Array(voice));
  return outPath;
};
