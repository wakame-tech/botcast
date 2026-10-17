import {
	VoiceForm,
	type VoiceInput,
	voiceInputToData,
} from "@/components/voice/VoiceForm";
import { $cms, CMS_TENANT, toRecordData } from "@/lib/cms_client";
import { createLazyFileRoute, useNavigate } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/voices/new")({
	component: NewVoice,
});

export function NewVoice() {
	const navigate = useNavigate();
	const newVoice = $cms.useMutation(
		"post",
		"/t/{tenantId}/records/{collectionId}",
	);

	const handleSubmit = async (input: VoiceInput) => {
		const record = await newVoice.mutateAsync({
			params: { path: { tenantId: CMS_TENANT, collectionId: "voices" } },
			body: { data: toRecordData(voiceInputToData(input)) },
		});
		// 参照音声・試聴用の音声は作成後の画面でアップロードする
		navigate({ to: "/voices/$voiceId", params: { voiceId: record.id } });
	};

	return (
		<>
			<h1>新しい声</h1>
			<p className="text-sm text-gray-500">
				参照音声・試聴用の音声は作成後にアップロードできます。参照音声は、権利と本人の同意があるものだけにしてください。
			</p>
			<VoiceForm onSubmit={handleSubmit} />
		</>
	);
}
