import { MailForm } from "@/components/mail/MailForm";
import { $api, type MailInput } from "@/lib/api_client";
import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/corners/$cornerId/newMail")({
	component: NewMail,
});

export default function NewMail() {
	const { cornerId } = Route.useParams();
	const getCorner = $api.useQuery("get", "/corners/{cornerId}", {
		params: { path: { cornerId } },
	});
	const newMail = $api.useMutation("post", "/corners/{cornerId}/mails");
	const onSubmit = (values: MailInput) => {
		newMail.mutate({
			params: {
				path: { cornerId },
			},
			body: {
				body: values.body,
			},
		});
	};

	if (!getCorner.data) {
		return null;
	}
	const corner = getCorner.data;

	return (
		<MailForm
			bodySchema={corner.mail_schema as Record<string, unknown>}
			onSubmit={onSubmit}
		/>
	);
}
