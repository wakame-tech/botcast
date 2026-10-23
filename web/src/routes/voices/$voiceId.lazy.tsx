import { Button } from "@/components/ui/button";
import { VoiceAudioField } from "@/components/voice/VoiceAudio";
import {
	VOICE_INPUT_KEYS,
	VoiceForm,
	type VoiceInput,
	voiceInputToData,
} from "@/components/voice/VoiceForm";
import { VoiceIconField } from "@/components/voice/VoiceIcon";
import { VoicePreview } from "@/components/voice/VoicePreview";
import { useSession } from "@/hooks/useSession";
import {
	$cms,
	CMS_TENANT,
	isOwner,
	recordToPodcast,
	recordToVoice,
	toRecordData,
} from "@/lib/cms_client";
import { createLazyFileRoute, useNavigate } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/voices/$voiceId")({
	component: VoiceDetail,
});

export function VoiceDetail() {
	const navigate = useNavigate();
	const { voiceId } = Route.useParams();
	const { userId } = useSession();
	const path = {
		tenantId: CMS_TENANT,
		collectionId: "voices",
		recordId: voiceId,
	};
	const { data: record, refetch } = $cms.useQuery(
		"get",
		"/t/{tenantId}/records/{collectionId}/{recordId}",
		{ params: { path } },
	);
	const updateVoice = $cms.useMutation(
		"put",
		"/t/{tenantId}/records/{collectionId}/{recordId}",
	);
	const { data: podcastRecords } = $cms.useQuery(
		"get",
		"/t/{tenantId}/records/{collectionId}",
		{ params: { path: { tenantId: CMS_TENANT, collectionId: "podcasts" } } },
	);
	const deleteVoice = $cms.useMutation(
		"delete",
		"/t/{tenantId}/records/{collectionId}/{recordId}",
	);

	if (!record) {
		return null;
	}
	const voice = recordToVoice(record);
	const editable = isOwner(voice, userId);
	// この声を配役に使っている番組 (削除すると音声生成が失敗する)
	const castIn = (podcastRecords ?? [])
		.map(recordToPodcast)
		.filter((p) => Object.values(p.cast).includes(voice.slug));

	const handleSubmit = async (input: VoiceInput) => {
		// data は丸ごと置き換わるため、ファイルのフィールド (ref_audio / sample / icon) を保って入れ直す
		const current = { ...((record.data ?? {}) as Record<string, unknown>) };
		for (const key of VOICE_INPUT_KEYS) {
			delete current[key];
		}
		await updateVoice.mutateAsync({
			params: { path },
			body: { data: toRecordData({ ...current, ...voiceInputToData(input) }) },
		});
		await refetch();
	};

	/** 試聴のテイクを参照音声にするとき、声の seed をそのテイクの seed にする (ファイルのフィールドは保つ) */
	const handleSetSeed = async (seed: number) => {
		const { data: latest } = await refetch();
		const current = {
			...(((latest ?? record).data ?? {}) as Record<string, unknown>),
		};
		await updateVoice.mutateAsync({
			params: { path },
			body: { data: toRecordData({ ...current, seed }) },
		});
		await refetch();
	};

	const handleDelete = async () => {
		await deleteVoice.mutateAsync({ params: { path } });
		navigate({ to: "/voices" });
	};

	return (
		<>
			<h1>{voice.name}</h1>
			<p className="text-sm text-gray-500">
				slug: {voice.slug} ・ エンジン: {voice.engine}
				{voice.voice_id && ` ・ voice_id: ${voice.voice_id}`}
			</p>
			{!voice.voice_id && !voice.ref_audio && (
				<p className="text-sm text-yellow-700">
					voice_id
					も参照音声も無いため、この声では音声を生成できません。参照音声をアップロードするか、voice_id
					を設定してください (irodori で参照音声を使わないなら <code>none</code>
					)
				</p>
			)}

			<VoiceIconField voiceId={voiceId} file={voice.icon} editable={editable} />
			<VoiceAudioField
				voiceId={voiceId}
				field="sample"
				label="試聴用の音声"
				file={voice.sample}
				editable={editable}
			/>
			<VoiceAudioField
				voiceId={voiceId}
				field="ref_audio"
				label="参照音声 (声の複製の元。OpenAI 互換のエンジンに同期する。あれば voice_id・caption より優先する。20〜30 秒以上のきれいな音声が安定する。権利と本人の同意があるものだけ)"
				file={voice.ref_audio}
				editable={editable}
			/>

			{editable && <VoicePreview voiceId={voiceId} onSetSeed={handleSetSeed} />}

			{editable && (
				<>
					{/* フォームは初期値を state に写すので、試聴のテイクで seed が変わったら作り直す
					    (古い seed のまま保存して、テイクの seed を戻してしまわないように) */}
					<VoiceForm
						key={voice.seed ?? "none"}
						value={voice}
						onSubmit={handleSubmit}
					/>
					{castIn.length > 0 && (
						<p className="text-sm text-red-500">
							この声は次の番組の配役に使われています。削除すると、配役を直すまで音声を生成できません:{" "}
							{castIn.map((p) => p.title).join("、")}
						</p>
					)}
					<div className="flex items-center">
						<div className="flex-grow" />
						<Button className="bg-red-400" onClick={handleDelete}>
							削除
						</Button>
					</div>
				</>
			)}
		</>
	);
}
