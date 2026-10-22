import { Button } from "@/components/ui/button";
import { useCmsFileUrl } from "@/hooks/useCmsFileUrl";
import {
	$cms,
	CMS_TENANT,
	type CmsFile,
	MAX_FILE_SIZE,
	audioContentType,
	fileToBase64,
	type imageContentType,
} from "@/lib/cms_client";
import { useQueryClient } from "@tanstack/react-query";
import { useRef, useState } from "react";

/** CMS に保存した音声を再生する (認証付きで取得して Blob URL にする) */
export function CmsAudio({ file }: { file?: CmsFile }) {
	const url = useCmsFileUrl(file?.url);
	if (!file) {
		return null;
	}
	if (!url) {
		return <span className="text-sm text-gray-500">読み込み中...</span>;
	}
	// biome-ignore lint/a11y/useMediaCaption: 試聴用の音声に字幕は無い
	return <audio controls src={url} />;
}

type UploadContentType =
	| NonNullable<ReturnType<typeof audioContentType>>
	| NonNullable<ReturnType<typeof imageContentType>>;

/** 声のファイルのフィールド */
export type VoiceFileField = "ref_audio" | "sample" | "icon";

/** 声のファイルのフィールドのアップロード・削除 (`/images/{field}`) */
export function useVoiceFile(voiceId: string, field: VoiceFileField) {
	const queryClient = useQueryClient();
	const upload = $cms.useMutation(
		"post",
		"/t/{tenantId}/records/{collectionId}/{recordId}/images/{fieldName}",
	);
	const remove = $cms.useMutation(
		"delete",
		"/t/{tenantId}/records/{collectionId}/{recordId}/images/{fieldName}",
	);
	const path = {
		tenantId: CMS_TENANT,
		collectionId: "voices",
		recordId: voiceId,
		fieldName: field,
	};
	// 声のレコード (ファイルのフィールドが書き換わる) を取り直す
	const refresh = () => queryClient.invalidateQueries();

	return {
		/** 上限を超える・形式が違うときはエラーの文言を返す */
		upload: async (
			file: File,
			contentType: UploadContentType | null,
			formatError: string,
		): Promise<string | undefined> => {
			if (!contentType) {
				return formatError;
			}
			if (file.size > MAX_FILE_SIZE) {
				return `ファイルが大きすぎます (上限 ${MAX_FILE_SIZE / 1024 / 1024}MB)`;
			}
			await upload.mutateAsync({
				params: { path },
				body: {
					data: await fileToBase64(file),
					content_type: contentType,
					filename: file.name,
				},
			});
			await refresh();
		},
		remove: async () => {
			await remove.mutateAsync({ params: { path } });
			await refresh();
		},
		isUploading: upload.isPending,
		isRemoving: remove.isPending,
	};
}

/** CMS に保存した音声の再生・停止のボタン (一覧に並べる小さい試聴用) */
export function CmsPlayButton({ file }: { file?: CmsFile }) {
	const url = useCmsFileUrl(file?.url);
	const audio = useRef<HTMLAudioElement>(null);
	const [playing, setPlaying] = useState(false);
	if (!file) {
		return null;
	}
	const toggle = () => {
		const el = audio.current;
		if (!el) return;
		if (el.paused) {
			el.currentTime = 0;
			el.play().catch(() => setPlaying(false));
		} else {
			el.pause();
		}
	};
	return (
		<>
			<Button
				type="button"
				variant="outline"
				size="sm"
				disabled={!url}
				onClick={toggle}
				aria-label={playing ? "試聴を停止" : "試聴"}
			>
				{playing ? "■ 停止" : "▶ 試聴"}
			</Button>
			{url && (
				// biome-ignore lint/a11y/useMediaCaption: 試聴用の音声に字幕は無い
				<audio
					ref={audio}
					src={url}
					onPlay={() => setPlaying(true)}
					onPause={() => setPlaying(false)}
					onEnded={() => setPlaying(false)}
				/>
			)}
		</>
	);
}

interface VoiceAudioFieldProps {
	voiceId: string;
	field: "ref_audio" | "sample";
	label: string;
	file?: CmsFile;
	editable: boolean;
}

/** 声の音声のフィールド (参照音声・試聴用の音声) の再生・アップロード・削除 */
export function VoiceAudioField(props: VoiceAudioFieldProps) {
	const voiceFile = useVoiceFile(props.voiceId, props.field);
	const [error, setError] = useState<string>();

	const handleFile = async (file: File) => {
		setError(undefined);
		setError(
			await voiceFile.upload(
				file,
				audioContentType(file),
				"wav か mp3 のファイルを選んでください",
			),
		);
	};

	return (
		<div className="flex flex-col gap-1 py-2">
			<span className="text-sm font-bold">{props.label}</span>
			{props.file ? (
				<CmsAudio file={props.file} />
			) : (
				<span className="text-sm text-gray-500">未設定</span>
			)}
			{props.editable && (
				<div className="flex gap-2 items-center">
					<input
						type="file"
						accept="audio/wav,audio/x-wav,audio/mpeg,.wav,.mp3"
						disabled={voiceFile.isUploading}
						onChange={(e) => {
							const file = e.target.files?.[0];
							e.target.value = "";
							if (file) {
								handleFile(file).catch((err) => setError(String(err)));
							}
						}}
					/>
					{props.file && (
						<Button
							type="button"
							className="bg-red-400"
							disabled={voiceFile.isRemoving}
							onClick={() =>
								voiceFile.remove().catch((err) => setError(String(err)))
							}
						>
							削除
						</Button>
					)}
				</div>
			)}
			{voiceFile.isUploading && (
				<span className="text-sm text-gray-500">アップロード中...</span>
			)}
			{error && <span className="text-sm text-red-500">{error}</span>}
		</div>
	);
}
