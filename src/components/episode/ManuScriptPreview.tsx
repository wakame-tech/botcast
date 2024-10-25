import { z } from "zod";

export const sectionSchema = z.discriminatedUnion("type", [
	z.object({
		type: z.literal("Serif"),
		speaker: z.string(),
		text: z.string(),
	}),
]);

export type Section = z.infer<typeof sectionSchema>;

export const manuscriptSchema = z.object({
	title: z.string(),
	sections: z.array(sectionSchema),
});

export type Manuscript = z.infer<typeof manuscriptSchema>;

export function ManuscriptPreview({ manuscript }: { manuscript: Manuscript }) {
	return (
		<div>
			<p className="font-bold text-lg">title: {manuscript.title}</p>
			{manuscript.sections.map((section) => (
				<p key={section.text}>
					<span className="pr-2 text-small text-gray">{section.speaker}</span>
					<span>{section.text}</span>
				</p>
			))}
		</div>
	);
}
