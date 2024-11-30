import type { Sections } from "@/trpc";

interface SectionsComponentProps {
	sections: Sections;
}

export function SectionsComponent(props: SectionsComponentProps) {
	return (
		<div>
			{props.sections.map((section) => (
				<p key={section.text}>
					<span className="pr-2 text-small text-gray">{section.speaker}</span>
					<span>{section.text}</span>
				</p>
			))}
		</div>
	);
}
