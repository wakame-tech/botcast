import { Button } from "@/components/ui/button";
import { VoiceAudioField } from "@/components/voice/VoiceAudio";
import {
	VOICE_INPUT_KEYS,
	VoiceForm,
	type VoiceInput,
	voiceInputToData,
} from "@/components/voice/VoiceForm";
import { useSession } from "@/hooks/useSession";
import {
	$cms,
	CMS_TENANT,
	isOwner,
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
	const deleteVoice = $cms.useMutation(
		"delete",
		"/t/{tenantId}/records/{collectionId}/{recordId}",
	);

	if (!record) {
		return null;
	}
	const voice = recordToVoice(record);
	const editable = isOwner(voice, userId);

	const handleSubmit = async (input: VoiceInput) => {
		// data は丸ごと置き換わるため、ファイルのフィールド (ref_audio / sample) を保って入れ直す
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
				label="参照音声 (声の複製の元。OpenAI 互換のエンジンに同期する。権利と本人の同意があるものだけ)"
				file={voice.ref_audio}
				editable={editable}
			/>

			{editable && (
				<>
					<VoiceForm
						key={record.updated_at ?? voiceId}
						value={voice}
						onSubmit={handleSubmit}
					/>
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
