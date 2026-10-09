import { z } from "zod";

export type SerifSection = {
	type: "Serif";
	speaker: string;
	text: string;
};

export type AudioSection = {
	type: "Audio";
	url: string;
	from?: number;
	to?: number;
};

export type Section = SerifSection | AudioSection;

export const PodcastInputSchema = z.object({
	icon: z.string().regex(/\p{Emoji_Presentation}/gu),
	title: z.string(),
	description: z.string(),
});

export type PodcastInput = z.infer<typeof PodcastInputSchema>;

export const ScriptInputSchema = z.object({
	title: z.string(),
	description: z.string(),
	template: z.string(),
});

export type ScriptInput = z.infer<typeof ScriptInputSchema>;

export const CornerInputSchema = z.object({
	title: z.string(),
	description: z.string().nullable(),
	requesting_mail: z.boolean(),
	mail_schema: z.string(),
});

export type CornerInput = z.infer<typeof CornerInputSchema>;

export const MailInputSchema = z.object({
	body: z.record(z.unknown()),
});

export type MailInput = z.infer<typeof MailInputSchema>;
