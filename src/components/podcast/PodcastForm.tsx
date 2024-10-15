import { Button } from "@/components/ui/button";
import {
	Form,
	FormControl,
	FormField,
	FormItem,
	FormLabel,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { z } from "zod";
import { MANUSCRIPT_TEMPLATE } from "../script/ScriptForm";

interface PodcastFormProps {
	values?: PodcastEditFormValues;
	onSubmit: (values: PodcastEditFormValues) => void;
}

const podcastEditFormSchema = z.object({
	title: z.string(),
	template: z.string(),
});

export type PodcastEditFormValues = z.infer<typeof podcastEditFormSchema>;

export function PodcastForm(props: PodcastFormProps) {
	const form = useForm<z.infer<typeof podcastEditFormSchema>>({
		resolver: zodResolver(podcastEditFormSchema),
		defaultValues: props.values ?? {
			title: "新しいポッドキャスト",
			template: JSON.stringify(MANUSCRIPT_TEMPLATE, null, 4),
		},
	});

	const onSubmit = async (values: PodcastEditFormValues) => {
		form.reset();
		props.onSubmit(values);
	};

	return (
		<Form {...form}>
			<form onSubmit={form.handleSubmit(onSubmit)} className="space-y-8">
				<FormField
					control={form.control}
					name="title"
					render={({ field }) => (
						<FormItem>
							<FormLabel>タイトル</FormLabel>
							<FormControl>
								<Input placeholder="title" {...field} />
							</FormControl>
						</FormItem>
					)}
				/>
				<FormField
					control={form.control}
					name="template"
					render={({ field }) => (
						<FormItem>
							<FormLabel>スクリプト</FormLabel>
							<FormControl>
								<Textarea rows={10} placeholder="template" {...field} />
							</FormControl>
						</FormItem>
					)}
				/>

				<Button type="submit">作成</Button>
			</form>
		</Form>
	);
}
