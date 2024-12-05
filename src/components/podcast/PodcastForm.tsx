import { Button } from "@/components/ui/button";
import {
	Form,
	FormControl,
	FormField,
	FormItem,
	FormLabel,
	FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { z } from "zod";

interface PodcastFormProps {
	values?: PodcastEditFormValues;
	onSubmit: (values: PodcastEditFormValues) => void;
}

const podcastEditFormSchema = z.object({
	title: z.string().min(1).max(50),
});

export type PodcastEditFormValues = z.infer<typeof podcastEditFormSchema>;

export function PodcastForm(props: PodcastFormProps) {
	const form = useForm<z.infer<typeof podcastEditFormSchema>>({
		resolver: zodResolver(podcastEditFormSchema),
		defaultValues: props.values ?? {
			title: "新しいポッドキャスト",
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
							<FormMessage />
						</FormItem>
					)}
				/>

				<Button disabled={!form.formState.isValid} type="submit">
					作成
				</Button>
			</form>
		</Form>
	);
}
