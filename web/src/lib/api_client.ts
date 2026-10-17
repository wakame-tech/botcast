import { z } from "zod";

export type SerifSection = {
	type: "Serif";
	/** 番組の配役の別名・声 (voices) の slug・旧形式の VOICEVOX のスタイル ID */
	speaker: string;
	text: string;
	/** 口調・感情の指定 (自由文) */
	style?: string;
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
