import { Episode, Series } from "../model.ts";
import R from "npm:remeda";
import { filterEmptyLine } from "../util.ts";
import tqdm from "npm:tqdm";
import { cached } from "../scrape.ts";
import { sha256 } from "https://denopkg.com/chiefbiiko/sha256@v1.0.0/mod.ts";
import { CACHE_DIR } from "../../../botcast-voicebox/src/index.ts";

const OPENAI_API_KEY = "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";

export interface Chat {
  model?: string;
  messages: ChatMessage[];
  temperature?: number;
}
export interface ChatMessage {
  role: "user" | "assistant";
  content: string;
}

export interface ErrorResponse {
  error: {
    message: string;
    type: string;
    param: string;
    code: "context_length_exceeded";
  };
}

export interface ChatResponse {
  id: string;
  object: string;
  created: number;
  model: string;
  usage: Usage;
  choices: ChoicesEntity[];
}

export interface Usage {
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
}

export interface ChoicesEntity {
  message: {
    content: string;
  };
  finish_reason: string;
  index: number;
}

export const generateResponse = async (
  apiKey: string,
  query: string
): Promise<string | null> => {
  const url = `https://api.openai.com/v1/chat/completions`;
  const chat: Chat = {
    model: "gpt-3.5-turbo-16k-0613",
    // model: "gpt-3.5-turbo-0613",
    messages: [
      {
        role: "user",
        content: query,
      },
    ],
  };
  const res = await fetch(url, {
    headers: {
      Authorization: `Bearer ${apiKey}`,
      "Content-Type": "application/json;charset=UTF-8",
    },
    method: "POST",
    body: JSON.stringify(chat),
  });
  const json: ChatResponse = await res.json();
  return json.choices?.[0].message.content ?? null;
};

const summerizeEpisode = async (episode: Episode): Promise<Episode> => {
  const prompt = `次の文章を10行程度で要約してください
  ---
  ${episode.serifs.map((serif) => serif.text).join("\n")}`;
  const resp = await cached(
    `${CACHE_DIR}/${sha256(prompt, "utf-8", "hex")}.txt`,
    async () => {
      let i = 0;
      while (i++ < 3) {
        const resp = await generateResponse(OPENAI_API_KEY, prompt);
        if (resp) {
          return resp;
        }
      }
      throw "unreachable";
    }
  );

  return {
    ...episode,
    serifs: R.pipe(
      resp.split("。"),
      filterEmptyLine,
      R.map((text) => ({ speaker: "", text }))
    ),
  };
};

export const summerizeSeries = async (series: Series): Promise<Series> => {
  const episodes = [];
  for (const episode of tqdm(series.episodes)) {
    episodes.push(await summerizeEpisode(episode));
  }
  return {
    ...series,
    episodes,
  };
};
