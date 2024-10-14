import { Button } from "@/components/ui/button";
import {
	Form,
	FormControl,
	FormField,
	FormItem,
	FormLabel,
} from "@/components/ui/form";
import { Textarea } from "@/components/ui/textarea";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { z } from "zod";

interface EpisodeFormProps {
	disabled?: boolean;
	episodeId: string;
	template: string;
	onSubmit: (values: EpisodeEditFormValues) => void;
}

const episodeEditFormSchema = z.object({
	template: z.string(),
});

export type EpisodeEditFormValues = z.infer<typeof episodeEditFormSchema>;

export function EpisodeForm(props: EpisodeFormProps) {
	const form = useForm<z.infer<typeof episodeEditFormSchema>>({
		resolver: zodResolver(episodeEditFormSchema),
		defaultValues: {
			template: props.template,
		},
	});

	return (
		<Form {...form}>
			<form onSubmit={form.handleSubmit(props.onSubmit)} className="space-y-8">
				<FormField
					control={form.control}
					name="template"
					render={({ field }) => (
						<FormItem>
							<FormLabel>スクリプト</FormLabel>
							<FormControl>
								<Textarea placeholder="template" {...field} />
							</FormControl>
						</FormItem>
					)}
				/>
				<Button disabled={props.disabled} type="submit">
					実行
				</Button>
			</form>
		</Form>
	);
}
