import type { Mail } from "@/lib/api_client";

interface MailListProps {
	mails: Mail[];
}

export function MailList(props: MailListProps) {
	return (
		<>
			{props.mails.map((mail) => (
				<div key={mail.id}>
					<MailListItem mail={mail} />
				</div>
			))}
		</>
	);
}

interface MailListItemProps {
	mail: Mail;
}

function MailListItem(props: MailListItemProps) {
	return (
		<ul className="">
			{Object.entries(props.mail.body).map(([k, v]) => {
				return (
					<li key={k} className="">
						<span className="pr-4 font-bold">{k}</span>
						<span className="">{JSON.stringify(v).replaceAll('"', "")}</span>
					</li>
				);
			})}
		</ul>
	);
}
