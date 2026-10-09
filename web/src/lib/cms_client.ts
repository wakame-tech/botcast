import { clearToken, getToken } from "@/hooks/useSession";
import createFetchClient, { type Middleware } from "openapi-fetch";
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

export interface Mail {
	id: string;
	podcast_id: string;
	episode_id: string;
	radio_name: string;
	body: string;
	user_id: string;
}

export function recordToMail(record: CmsRecord): Mail {
	const d = record.data as Record<string, unknown>;
	return {
		id: record.id,
		podcast_id: d.podcast_id as string,
		episode_id: d.episode_id as string,
		radio_name: (d.radio_name as string) ?? "",
		body: (d.body as string) ?? "",
		user_id: d.user_id as string,
	};
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

export const CMS_URL = import.meta.env.VITE_CMS_URL ?? "http://localhost:3002";

const authMiddleware: Middleware = {
	async onRequest({ request }) {
		const token = getToken();
		if (token) {
			request.headers.set("Authorization", `Bearer ${token}`);
		}
		return request;
	},
	async onResponse({ response }) {
		// トークン失効時はサインイン画面へ戻す
		if (response.status === 401) {
			clearToken();
			if (window.location.pathname !== "/signin") {
				window.location.assign("/signin");
			}
		}
		return response;
	},
};

// botcast-cms の collection ID は自動採番のため、名前 → ID の対応を引いて解決する
let collectionIds: Promise<Map<string, string>> | null = null;

const fetchCollectionIds = async (): Promise<Map<string, string>> => {
	const token = getToken();
	const res = await fetch(`${CMS_URL}/collections`, {
		headers: token ? { Authorization: `Bearer ${token}` } : {},
	});
	if (!res.ok) {
		throw new Error(`failed to list collections: ${res.status}`);
	}
	const collections = (await res.json()) as { id: string; name: string }[];
	return new Map(
		collections.map((c) => [c.name, c.id.replace(/^collection:/, "")]),
	);
};

const resolveCollectionId = async (name: string): Promise<string> => {
	collectionIds ??= fetchCollectionIds();
	let ids = await collectionIds.catch(() => null);
	if (!ids?.has(name)) {
		// 取得失敗・後から作られた collection に備えて一度だけ取り直す
		collectionIds = fetchCollectionIds();
		ids = await collectionIds.catch(() => null);
	}
	return ids?.get(name) ?? name;
};

const collectionNameMiddleware: Middleware = {
	async onRequest({ request, schemaPath }) {
		if (!schemaPath.startsWith("/records/{collectionId}")) {
			return request;
		}
		const url = new URL(request.url);
		const match = url.pathname.match(/^(.*\/records\/)([^/]+)(.*)$/);
		if (!match) {
			return request;
		}
		const [, prefix = "", name = "", rest = ""] = match;
		const id = await resolveCollectionId(decodeURIComponent(name));
		url.pathname = `${prefix}${encodeURIComponent(id)}${rest}`;
		// body をストリームのまま引き継ぐと送信に失敗するため、Blob に読み出して作り直す
		const hasBody = request.method !== "GET" && request.method !== "HEAD";
		return new Request(url, {
			method: request.method,
			headers: request.headers,
			body: hasBody ? await request.blob() : undefined,
			signal: request.signal,
		});
	},
};

const fetchClient = createFetchClient<paths>({
	baseUrl: CMS_URL,
});
fetchClient.use(authMiddleware);
fetchClient.use(collectionNameMiddleware);

export const $cms = createClient(fetchClient);

/** CMS に保存されたファイル (`/records/{c}/{r}/images/{field}`) を取得する */
export const fetchCmsFile = async (path: string): Promise<Blob> => {
	const token = getToken();
	const res = await fetch(`${CMS_URL}${path}`, {
		headers: token ? { Authorization: `Bearer ${token}` } : {},
	});
	if (!res.ok) {
		throw new Error(`Failed to fetch ${path}: ${res.status}`);
	}
	// CMS はファイル本体を Base64 の JSON で返す
	const { data, content_type } = (await res.json()) as {
		data: string;
		content_type: string;
	};
	const bytes = Uint8Array.from(atob(data), (c) => c.charCodeAt(0));
	return new Blob([bytes], { type: content_type });
};
