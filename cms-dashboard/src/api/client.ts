// @specre 01KNM2BBT598J55K9BMM975FCZ
// @specre 01KNM2BBT5T43CMWAX32H3K14G
// @specre 01KNM91QXA84F7HTBYSKM5TZMP
// @specre 01KNM91QXWGPMAEZZE0M7GCTB4
// @specre 01KNVEV4DB8QKB8DYKJQHYKDQD
import { getStoredToken } from "../contexts/AuthContext";
import type {
	CanvasQueryRequest,
	CanvasQueryResponse,
	Collection,
	CopyRecordRequest,
	CreateEdgeRequest,
	CreateOrUpdateCollection,
	CreateOrUpdateJob,
	CreateOrUpdateRecord,
	Edge,
	ExecuteScriptRequest,
	ExecuteScriptResponse,
	GenerateRecordRequest,
	GetImageResponse,
	ImageInfo,
	Job,
	MergeRecordsRequest,
	RecordWithRelations,
	Record_,
	SplitRecordRequest,
	SplitRecordResponse,
	UploadImageRequest,
} from "../types";

const BASE_URL = "";

/**
 * SurrealDB は ID を "table:key" 形式で返す。
 * API パスパラメータには "key" 部分のみ使用する。
 * 例: "collection:abc123" -> "abc123"
 */
export function rawId(id: string): string {
	const colonIdx = id.indexOf(":");
	return colonIdx !== -1 ? id.slice(colonIdx + 1) : id;
}

async function request<T>(path: string, options?: RequestInit): Promise<T> {
	const token = getStoredToken();
	const authHeader: Record<string, string> = token
		? { Authorization: `Bearer ${token}` }
		: {};
	const res = await fetch(`${BASE_URL}${path}`, {
		headers: { "Content-Type": "application/json", ...authHeader },
		...options,
	});
	if (!res.ok) {
		const text = await res.text();
		throw new Error(`${res.status} ${res.statusText}: ${text}`);
	}
	if (res.status === 204) return undefined as unknown as T;
	return res.json() as Promise<T>;
}

// Collections
export const collectionsApi = {
	list: () => request<Collection[]>("/collections"),
	read: (id: string) => request<Collection>(`/collections/${rawId(id)}`),
	create: (body: CreateOrUpdateCollection) =>
		request<Collection>("/collections", {
			method: "POST",
			body: JSON.stringify(body),
		}),
	update: (id: string, name: string) =>
		request<Collection>(`/collections/${rawId(id)}`, {
			method: "PUT",
			body: JSON.stringify({ name }),
		}),
	delete: (id: string) =>
		request<void>(`/collections/${rawId(id)}`, { method: "DELETE" }),
};

// Records
export const recordsApi = {
	list: (collectionId: string) =>
		request<Record_[]>(`/records/${rawId(collectionId)}`),
	read: (collectionId: string, recordId: string) =>
		request<Record_>(`/records/${rawId(collectionId)}/${rawId(recordId)}`),
	create: (collectionId: string, body: CreateOrUpdateRecord) =>
		request<Record_>(`/records/${rawId(collectionId)}`, {
			method: "POST",
			body: JSON.stringify(body),
		}),
	update: (
		collectionId: string,
		recordId: string,
		body: CreateOrUpdateRecord,
	) =>
		request<Record_>(`/records/${rawId(collectionId)}/${rawId(recordId)}`, {
			method: "PUT",
			body: JSON.stringify(body),
		}),
	delete: (collectionId: string, recordId: string) =>
		request<void>(`/records/${rawId(collectionId)}/${rawId(recordId)}`, {
			method: "DELETE",
		}),
	generate: (collectionId: string, body: GenerateRecordRequest) =>
		request<Record_>(`/records/${rawId(collectionId)}/generate`, {
			method: "POST",
			body: JSON.stringify(body),
		}),
	listWithRelations: (collectionId: string) =>
		request<RecordWithRelations[]>(
			`/records/${rawId(collectionId)}/with-relations`,
		),
	copy: (collectionId: string, recordId: string, body: CopyRecordRequest) =>
		request<Record_>(
			`/records/${rawId(collectionId)}/${rawId(recordId)}/copy`,
			{ method: "POST", body: JSON.stringify(body) },
		),
	merge: (collectionId: string, body: MergeRecordsRequest) =>
		request<Record_>(`/records/${rawId(collectionId)}/merge`, {
			method: "POST",
			body: JSON.stringify(body),
		}),
	split: (collectionId: string, recordId: string, body: SplitRecordRequest) =>
		request<SplitRecordResponse>(
			`/records/${rawId(collectionId)}/${rawId(recordId)}/split`,
			{ method: "POST", body: JSON.stringify(body) },
		),
};

// Images
export const imagesApi = {
	upload: (
		collectionId: string,
		recordId: string,
		fieldName: string,
		body: UploadImageRequest,
	) =>
		request<ImageInfo>(
			`/records/${rawId(collectionId)}/${rawId(recordId)}/images/${encodeURIComponent(fieldName)}`,
			{ method: "POST", body: JSON.stringify(body) },
		),
	get: (collectionId: string, recordId: string, fieldName: string) =>
		request<GetImageResponse>(
			`/records/${rawId(collectionId)}/${rawId(recordId)}/images/${encodeURIComponent(fieldName)}`,
		),
	delete: (collectionId: string, recordId: string, fieldName: string) =>
		request<void>(
			`/records/${rawId(collectionId)}/${rawId(recordId)}/images/${encodeURIComponent(fieldName)}`,
			{ method: "DELETE" },
		),
};

// Scripts
export const scriptsApi = {
	execute: (body: ExecuteScriptRequest) =>
		request<ExecuteScriptResponse>("/scripts", {
			method: "POST",
			body: JSON.stringify(body),
		}),
};

// Jobs
export const jobsApi = {
	list: () => request<Job[]>("/jobs"),
	create: (body: CreateOrUpdateJob) =>
		request<void>("/jobs", {
			method: "POST",
			body: JSON.stringify(body),
		}),
};

// Canvas
export const canvasApi = {
	listEdges: (recordId: string) =>
		request<Edge[]>(`/canvas/edges?record_id=${encodeURIComponent(recordId)}`),
	createEdge: (body: CreateEdgeRequest) =>
		request<Edge>("/canvas/edges", {
			method: "POST",
			body: JSON.stringify(body),
		}),
	deleteEdge: (edgeId: string) =>
		request<void>(`/canvas/edges/${encodeURIComponent(edgeId)}`, {
			method: "DELETE",
		}),
	query: (body: CanvasQueryRequest) =>
		request<CanvasQueryResponse>("/canvas/query", {
			method: "POST",
			body: JSON.stringify(body),
		}),
};
