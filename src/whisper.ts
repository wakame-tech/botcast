const openAIApiKey = Deno.env.get("OPENAI_API_KEY")!;

interface VerboseJson {
  task: "transcribe";
  language: string;
  duration: number;
  segments: Segment[];
  text: string;
}

interface Segment {
  id: number;
  seek: number;
  start: number;
  end: number;
  text: string;
}

export const transcribe = async (file: File): Promise<string> => {
  const form = new FormData();
  form.set("response_format", "verbose_json");
  form.set("model", "whisper-1");
  form.set("file", file);
  const url = `https://api.openai.com/v1/audio/transcriptions`;
  const res: VerboseJson = await fetch(url, {
    headers: {
      Authorization: `Bearer ${openAIApiKey}`,
    },
    method: "POST",
    body: form,
  })
    .then((res) => res.json())
    .catch((e) => console.log(e));
  if (!res) {
    throw "error";
  }
  console.log(res);
  return res.text;
};

const wav = await Deno.readFile("aspara.mp3");
await transcribe(new File([wav], "a.wav"));
