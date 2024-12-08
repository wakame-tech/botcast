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
import { PodcastInputSchema } from "@/trpc.ts";
import type { PodcastInput } from "@/trpc.ts";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";

interface PodcastFormProps {
	values?: PodcastInput;
	onSubmit: (values: PodcastInput) => void;
}

export function PodcastForm(props: PodcastFormProps) {
	const form = useForm<PodcastInput>({
		resolver: zodResolver(PodcastInputSchema),
		defaultValues:
			props.values ??
			({
				icon: "🎧",
				title: "新しいポッドキャスト",
				description: "",
			} satisfies PodcastInput),
	});

	const onSubmit = async (values: PodcastInput) => {
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

				<FormField
					control={form.control}
					name="description"
					render={({ field }) => (
						<FormItem>
							<FormLabel>説明</FormLabel>
							<FormControl>
								<Input
									placeholder="description"
									{...field}
									value={field.value ?? undefined}
								/>
							</FormControl>
							<FormMessage />
						</FormItem>
					)}
				/>

				<Button disabled={!form.formState.isValid} type="submit">
					{props.values ? "更新" : "作成"}
				</Button>
			</form>
		</Form>
	);
}
