import { clearToken, getToken } from "@/hooks/useSession";
import createFetchClient, { type Middleware } from "openapi-fetch";
import createClient from "openapi-react-query";
import type { Section } from "./api_client";
import type { components, paths } from "./cms_api";

export type CmsRecord = components["schemas"]["Record_"];

export interface PodcastSchedule {
	/** Quartz 形式 7 フィールド (秒 分 時 日 月 曜日 年)、UTC */
	cron: string;
	script_id: string;
	enabled: boolean;
	/** スクリプトに渡す引数の値 (スクリプトの `arguments` JSON Schema に従う) */
	arguments?: Record<string, unknown>;
}

/** 配役: 台本の `speaker` に書く別名 → 声 (voices) の slug */
export type Cast = Record<string, string>;

export interface Podcast {
	id: string;
	title: string;
	icon: string;
	description: string;
	/** 所有者 (JWT の sub)。CMS のレコードの owner_id */
	owner_id: string | null;
	schedule?: PodcastSchedule;
	cast: Cast;
}

/** CMS に保存したファイルのフィールド (`/images/{field}` のアップロードで CMS が書き込む) */
export interface CmsFile {
	pointer: string;
	content_type: string;
	size: number;
	url: string;
}

export interface VoiceParams {
	speed?: number;
	pitch?: number;
	intonation?: number;
	volume?: number;
	/** エンジンごとの素通しの値 */
	extra?: Record<string, unknown>;
}

/** 声 (話者) のマスタ。worker の TTS エンジンの設定名 `engine` と、エンジンの声 `voice_id` を持つ */
export interface Voice {
	id: string;
	slug: string;
	name: string;
	engine: string;
	/** VOICEVOX 互換はスタイル ID、OpenAI 互換は voice 名。参照音声を使う場合は省略できる */
	voice_id?: string;
	/** VOICEVOX 互換で使う、style の名前 → スタイル ID */
	styles?: Record<string, string>;
	ref_audio?: CmsFile;
	params?: VoiceParams;
	seed?: number;
	caption?: string;
	/** 試聴用の音声 */
	sample?: CmsFile;
	owner_id: string | null;
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
	owner_id: string | null;
}

export interface Script {
	id: string;
	title: string;
	description: string;
	template: string;
	arguments: Record<string, unknown>;
	owner_id: string | null;
}

export interface Mail {
	id: string;
	episode_id: string;
	radio_name: string;
	body: string;
	owner_id: string | null;
}

export function recordToMail(record: CmsRecord): Mail {
	const d = record.data as Record<string, unknown>;
	return {
		id: record.id,
		episode_id: d.episode_id as string,
		radio_name: (d.radio_name as string) ?? "",
		body: (d.body as string) ?? "",
		owner_id: record.owner_id,
	};
}

export function recordToPodcast(record: CmsRecord): Podcast {
	const d = record.data as Record<string, unknown>;
	return {
		id: record.id,
		title: d.title as string,
		icon: d.icon as string,
		description: (d.description as string) ?? "",
		owner_id: record.owner_id,
		schedule: d.schedule as PodcastSchedule | undefined,
		cast: (d.cast as Cast | undefined) ?? {},
	};
}

export function recordToVoice(record: CmsRecord): Voice {
	const d = record.data as Record<string, unknown>;
	return {
		id: record.id,
		slug: d.slug as string,
		name: d.name as string,
		engine: d.engine as string,
		voice_id: d.voice_id as string | undefined,
		styles: d.styles as Record<string, string> | undefined,
		ref_audio: d.ref_audio as CmsFile | undefined,
		params: d.params as VoiceParams | undefined,
		seed: d.seed as number | undefined,
		caption: d.caption as string | undefined,
		sample: d.sample as CmsFile | undefined,
		owner_id: record.owner_id,
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
		owner_id: record.owner_id,
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
		owner_id: record.owner_id,
	};
}

/** サインイン中のユーザーがレコードの所有者か（編集・削除の導線を出すか） */
export function isOwner(
	record: { owner_id: string | null },
	userId: string | null,
): boolean {
	return userId !== null && record.owner_id === userId;
}

export function toRecordData(
	data: Record<string, unknown>,
): Record<string, never> {
	return data as unknown as Record<string, never>;
}

export const CMS_URL = import.meta.env.VITE_CMS_URL ?? "http://localhost:3002";

/**
 * botcast-cms のテナント。CMS はマルチテナントで、全エンドポイントが
 * `/t/{tenantId}/...` 配下にある。botcast が話す相手は常に自分のテナント 1 つ。
 */
export const CMS_TENANT = import.meta.env.VITE_CMS_TENANT ?? "botcast";

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
	const res = await fetch(`${CMS_URL}/t/${CMS_TENANT}/collections`, {
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
		if (!schemaPath.startsWith("/t/{tenantId}/records/{collectionId}")) {
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

/** CMS が受け付ける音声の MIME type (`audio/x-wav` などのブラウザの表記をそろえる) */
export const audioContentType = (
	file: File,
): "audio/wav" | "audio/mpeg" | null => {
	const type = file.type.toLowerCase();
	const name = file.name.toLowerCase();
	if (/wav/.test(type) || name.endsWith(".wav")) return "audio/wav";
	if (/mpeg|mp3/.test(type) || name.endsWith(".mp3")) return "audio/mpeg";
	return null;
};

/** アップロードするファイルの大きさの上限 (worker の `MAX_FILE_SIZE` と同じ値) */
export const MAX_FILE_SIZE = 20 * 1024 * 1024;

/** ファイルを Base64 にする (CMS のアップロードの形式) */
export const fileToBase64 = async (file: Blob): Promise<string> => {
	const bytes = new Uint8Array(await file.arrayBuffer());
	let binary = "";
	for (let i = 0; i < bytes.length; i += 0x8000) {
		binary += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
	}
	return btoa(binary);
};

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
