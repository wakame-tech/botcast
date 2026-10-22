import { Button } from "@/components/ui/button";
import { useVoiceFile } from "@/components/voice/VoiceAudio";
import { useCmsFileUrl } from "@/hooks/useCmsFileUrl";
import { type CmsFile, imageContentType } from "@/lib/cms_client";
import { cn } from "@/lib/utils";
import { useState } from "react";

/** 声のアイコン。未設定 (や読み込み中) は既定の絵 */
export function VoiceIcon({
	file,
	className,
}: { file?: CmsFile; className?: string }) {
	const url = useCmsFileUrl(file?.url);
	return (
		<span
			className={cn(
				"w-8 h-8 rounded-full overflow-hidden bg-teal-100 inline-flex items-center justify-center shrink-0",
				className,
			)}
		>
			{url ? (
				<img src={url} alt="" className="w-full h-full object-cover" />
			) : (
				<span aria-hidden>🗣️</span>
			)}
		</span>
	);
}

interface VoiceIconFieldProps {
	voiceId: string;
	file?: CmsFile;
	editable: boolean;
}

/** 声のアイコンの表示・アップロード・削除 */
export function VoiceIconField(props: VoiceIconFieldProps) {
	const voiceFile = useVoiceFile(props.voiceId, "icon");
	const [error, setError] = useState<string>();

	const handleFile = async (file: File) => {
		setError(undefined);
		setError(
			await voiceFile.upload(
				file,
				imageContentType(file),
				"png・jpeg・webp・gif の画像を選んでください",
			),
		);
	};

	return (
		<div className="flex flex-col gap-1 py-2">
			<span className="text-sm font-bold">アイコン</span>
			<VoiceIcon file={props.file} className="w-16 h-16 text-3xl" />
			{props.editable && (
				<div className="flex gap-2 items-center">
					<input
						type="file"
						accept="image/png,image/jpeg,image/webp,image/gif"
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
