import { MailForm } from "@/components/mail/MailForm";
import type { MailInput } from "@/lib/api_client";
import { $api } from "@/lib/api_client";
import { $cms } from "@/lib/cms_client";
import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/corners/$cornerId/newMail")({
	component: NewMail,
});

export default function NewMail() {
	const { cornerId } = Route.useParams();
	const getCorner = $api.useQuery("get", "/corners/{cornerId}", {
		params: { path: { cornerId } },
	});
	const corner = getCorner.data;
	const collectionId = corner?.cms_collection_id ?? "";

	const createRecord = $cms.useMutation("post", "/records/{collectionId}");
	const onSubmit = (values: MailInput) => {
		if (!collectionId) return;
		createRecord.mutate({
			params: { path: { collectionId } },
			body: { data: values.body as Record<string, never> },
		});
	};

	if (!corner) {
		return null;
	}

	return (
		<MailForm
			bodySchema={corner.mail_schema as Record<string, unknown>}
			onSubmit={onSubmit}
		/>
	);
}
