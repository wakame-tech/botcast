import { MailForm } from "@/components/mail/MailForm";
import { type MailInput, trpc } from "@/trpc";
import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/corners/$cornerId/newMail")({
	component: NewMail,
});

export default function NewMail() {
	const { cornerId } = Route.useParams();
	const getCorner = trpc.corner.useQuery({ id: cornerId });
	const newMail = trpc.newMail.useMutation();
	const onSubmit = (values: MailInput) => {
		newMail.mutate({ cornerId, body: values.body });
	};

	if (!getCorner.data) {
		return null;
	}
	const corner = getCorner.data.corner;

	return (
		<MailForm
			bodySchema={corner.mail_schema as Record<string, unknown>}
			onSubmit={onSubmit}
		/>
	);
}
