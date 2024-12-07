import type { Sections } from "@/trpc";

interface SectionsComponentProps {
	sections: Sections;
}

export function SectionsComponent(props: SectionsComponentProps) {
	return (
		<>
			{props.sections.map((section) => (
				<div key={section.type === "Serif" ? section.text : section.url}>
					{section.type === "Serif" && (
						<p>
							<span className="pr-2 text-small text-gray">
								{section.speaker}
							</span>
							<span>{section.text}</span>
						</p>
					)}
					{section.type === "Audio" && (
						<p>
							<span className="pr-2 text-small text-gray">♪</span>
						</p>
					)}
				</div>
			))}
		</>
	);
}
