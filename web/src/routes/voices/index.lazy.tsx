import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { CmsAudio } from "@/components/voice/VoiceAudio";
import { VoiceIcon } from "@/components/voice/VoiceIcon";
import { $cms, CMS_TENANT, recordToVoice } from "@/lib/cms_client";
import { Link, createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/voices/")({
	component: Voices,
});

/** 声 (話者) の一覧。番組の配役で選ぶ */
export default function Voices() {
	const { data: records } = $cms.useQuery(
		"get",
		"/t/{tenantId}/records/{collectionId}",
		{
			params: { path: { tenantId: CMS_TENANT, collectionId: "voices" } },
		},
	);
	const voices = (records ?? []).map(recordToVoice);

	return (
		<div>
			<div className="pb-2 flex items-center">
				<div className="flex-grow" />
				<Link to="/voices/new">
					<Button>新規作成</Button>
				</Link>
			</div>

			{voices.map((voice) => (
				<div key={voice.id} className="p-2">
					<Card>
						<CardHeader>
							<CardTitle className="flex items-center gap-2">
								<VoiceIcon file={voice.icon} className="w-10 h-10" />
								<Link
									to="/voices/$voiceId"
									params={{ voiceId: voice.id }}
									className="no-underline"
								>
									<span className="text-xl font-bold">{voice.name}</span>
								</Link>
								<span className="text-sm text-gray-500">
									{voice.slug} ・ {voice.engine}
								</span>
							</CardTitle>
						</CardHeader>
						<CardContent>
							{voice.caption}
							<CmsAudio file={voice.sample} />
						</CardContent>
					</Card>
				</div>
			))}
		</div>
	);
}
