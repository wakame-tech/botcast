import { Button } from "@/components/ui/button";
import { useCmsFileUrl } from "@/hooks/useCmsFileUrl";
import {
	$cms,
	CMS_TENANT,
	type CmsFile,
	audioContentType,
	fileToBase64,
} from "@/lib/cms_client";
import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

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

interface VoiceAudioFieldProps {
	voiceId: string;
	field: "ref_audio" | "sample";
	label: string;
	file?: CmsFile;
	editable: boolean;
}

/** 声の音声のフィールド (参照音声・試聴用の音声) の再生・アップロード・削除 */
export function VoiceAudioField(props: VoiceAudioFieldProps) {
	const queryClient = useQueryClient();
	const upload = $cms.useMutation(
		"post",
		"/t/{tenantId}/records/{collectionId}/{recordId}/images/{fieldName}",
	);
	const remove = $cms.useMutation(
		"delete",
		"/t/{tenantId}/records/{collectionId}/{recordId}/images/{fieldName}",
	);
	const [error, setError] = useState<string>();
	const path = {
		tenantId: CMS_TENANT,
		collectionId: "voices",
		recordId: props.voiceId,
		fieldName: props.field,
	};
	// 声のレコード (ファイルのフィールドが書き換わる) を取り直す
	const refresh = () => queryClient.invalidateQueries();

	const handleFile = async (file: File) => {
		const contentType = audioContentType(file);
		if (!contentType) {
			setError("wav か mp3 のファイルを選んでください");
			return;
		}
		setError(undefined);
		await upload.mutateAsync({
			params: { path },
			body: {
				data: await fileToBase64(file),
				content_type: contentType,
				filename: file.name,
			},
		});
		await refresh();
	};

	const handleRemove = async () => {
		await remove.mutateAsync({ params: { path } });
		await refresh();
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
						disabled={upload.isPending}
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
							disabled={remove.isPending}
							onClick={() =>
								handleRemove().catch((err) => setError(String(err)))
							}
						>
							削除
						</Button>
					)}
				</div>
			)}
			{upload.isPending && (
				<span className="text-sm text-gray-500">アップロード中...</span>
			)}
			{error && <span className="text-sm text-red-500">{error}</span>}
		</div>
	);
}
