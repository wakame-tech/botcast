import createFetchClient from "openapi-fetch";
import createClient from "openapi-react-query";
import type { Section } from "./api_client";
import type { components, paths } from "./cms_api";

export type CmsRecord = components["schemas"]["Record_"];

export interface Podcast {
	id: string;
	title: string;
	icon: string;
	description: string;
	user_id: string;
}

export interface Episode {
	id: string;
	title: string;
	podcast_id: string;
	description: string;
	audio_url?: string;
	srt_url?: string;
	sections: Section[];
	duration_sec?: number;
	user_id: string;
}

export interface Script {
	id: string;
	title: string;
	description: string;
	template: string;
	arguments: Record<string, unknown>;
	user_id: string;
}

export function recordToPodcast(record: CmsRecord): Podcast {
	const d = record.data as Record<string, unknown>;
	return {
		id: record.id,
		title: d.title as string,
		icon: d.icon as string,
		description: (d.description as string) ?? "",
		user_id: d.user_id as string,
	};
}

export function recordToEpisode(record: CmsRecord): Episode {
	const d = record.data as Record<string, unknown>;
	return {
		id: record.id,
		title: d.title as string,
		podcast_id: d.podcast_id as string,
		description: (d.description as string) ?? "",
		audio_url: d.audio_url as string | undefined,
		srt_url: d.srt_url as string | undefined,
		sections: (d.sections as Section[]) ?? [],
		duration_sec: d.duration_sec as number | undefined,
		user_id: d.user_id as string,
	};
}

export function recordToScript(record: CmsRecord): Script {
	const d = record.data as Record<string, unknown>;
	return {
		id: record.id,
		title: d.title as string,
		description: (d.description as string) ?? "",
		template: (d.template as string) ?? "",
		arguments: (d.arguments as Record<string, unknown>) ?? {},
		user_id: d.user_id as string,
	};
}

export function toRecordData(
	data: Record<string, unknown>,
): Record<string, never> {
	return data as unknown as Record<string, never>;
}

const fetchClient = createFetchClient<paths>({
	baseUrl: import.meta.env.VITE_CMS_API_URL ?? "http://localhost:3002",
});

export const $cms = createClient(fetchClient);
