import { getToken } from "@/hooks/useSession";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

export const WORKER_URL =
	import.meta.env.VITE_WORKER_URL ?? "http://localhost:9001";

/** worker の `POST /jobs` の body（worker の `Args` と同じ形式） */
export type JobArgs =
	| { type: "generateAudio"; episodeId: string }
	| {
			type: "generateScript";
			episodeId: string;
			prompt: string;
			/** 台本の生成に続けて同じジョブで音声も生成する */
			generateAudio?: boolean;
	  }
	| { type: "generateEpisode"; podcastId: string };

/** LLM のトークン数の合計（worker の `Usage`） */
export interface TokenUsage {
	inputTokens: number;
	outputTokens: number;
	cacheCreationInputTokens?: number | null;
	cacheReadInputTokens?: number | null;
}

export interface JobInfo {
	id: string;
	name: string;
	status: string;
	traceId?: string;
	/** LLM を呼んだジョブのトークン数の合計 */
	usage?: TokenUsage;
}

const workerUrl = (path: string) => `${WORKER_URL.replace(/\/$/, "")}${path}`;

/** ジョブを投入する。認証は botcast-cms のトークンをそのまま渡し、worker 側で CMS に検証させる */
export const enqueueJob = async (args: JobArgs): Promise<void> => {
	const token = getToken();
	const res = await fetch(workerUrl("/jobs"), {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
			...(token ? { Authorization: `Bearer ${token}` } : {}),
		},
		body: JSON.stringify(args),
	});
	if (!res.ok) {
		throw new Error(`ジョブの投入に失敗しました (${res.status})`);
	}
};

/** 声の試聴の 1 テイク（worker の `PreviewTake`） */
export interface VoicePreviewTake {
	/** このテイクの seed。参照音声にするときに声の seed にする */
	seed: number;
	contentType: string;
	audioBase64: string;
	elapsedMs: number;
}

export interface VoicePreviewResponse {
	voiceId: string;
	engine: string;
	text: string;
	takes: VoicePreviewTake[];
}

/** 試聴の本文の上限（worker の `MAX_PREVIEW_TEXT_CHARS`） */
export const MAX_PREVIEW_TEXT_CHARS = 600;
/** テイク数の上限（worker の `MAX_PREVIEW_TAKES`） */
export const MAX_PREVIEW_TAKES = 4;

/**
 * 声の試聴を作る（`POST /voices/{voiceId}/previews`。声の持ち主だけ）。
 * text を省略すると worker の既定の本文（数文）になる。音声は保存されない
 */
export const createVoicePreviews = async (
	voiceId: string,
	body: { text?: string; count?: number },
	signal?: AbortSignal,
): Promise<VoicePreviewResponse> => {
	const token = getToken();
	const res = await fetch(
		workerUrl(`/voices/${encodeURIComponent(voiceId)}/previews`),
		{
			method: "POST",
			headers: {
				"Content-Type": "application/json",
				...(token ? { Authorization: `Bearer ${token}` } : {}),
			},
			body: JSON.stringify(body),
			signal,
		},
	);
	if (!res.ok) {
		const detail = await res
			.json()
			.then((b: { error?: string }) => b.error)
			.catch(() => undefined);
		throw new Error(
			`試聴の生成に失敗しました (${res.status}${detail ? `: ${detail}` : ""})`,
		);
	}
	return (await res.json()) as VoicePreviewResponse;
};

export const fetchJobs = async (): Promise<JobInfo[]> => {
	const res = await fetch(workerUrl("/jobs"));
	if (!res.ok) {
		throw new Error(`ジョブ一覧の取得に失敗しました (${res.status})`);
	}
	return (await res.json()) as JobInfo[];
};

const JOBS_QUERY_KEY = ["worker", "jobs"];

export const useJobs = () =>
	useQuery({
		queryKey: JOBS_QUERY_KEY,
		queryFn: fetchJobs,
		refetchInterval: 5000,
	});

export const useEnqueueJob = () => {
	const queryClient = useQueryClient();
	return useMutation({
		mutationFn: enqueueJob,
		onSuccess: () =>
			queryClient.invalidateQueries({ queryKey: JOBS_QUERY_KEY }),
	});
};
