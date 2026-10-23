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
