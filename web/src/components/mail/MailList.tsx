interface MailListProps {
	records: Record<string, unknown>[];
}

export function MailList(props: MailListProps) {
	return (
		<>
			{props.records.map((record) => {
				const key = (record.id as string) ?? JSON.stringify(record);
				return (
					<div key={key}>
						<MailListItem record={record} />
					</div>
				);
			})}
		</>
	);
}

interface MailListItemProps {
	record: Record<string, unknown>;
}

function MailListItem(props: MailListItemProps) {
	return (
		<ul className="">
			{Object.entries(props.record).map(([k, v]) => {
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
