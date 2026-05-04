import { Button } from "@/components/ui/button";
import {
	Form,
	FormControl,
	FormField,
	FormItem,
	FormLabel,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { z } from "zod";

const EvaluateScriptInputSchema = z.object({
	scriptId: z.string(),
	cron: z.string(),
});

export type EvaluateScriptInput = z.infer<typeof EvaluateScriptInputSchema>;

interface EvaluateScriptFormProps {
	disabled?: boolean;
	onSubmit: (values: EvaluateScriptInput) => void;
}

export function EvaluateScriptForm(props: EvaluateScriptFormProps) {
	const form = useForm<EvaluateScriptInput>({
		resolver: zodResolver(EvaluateScriptInputSchema),
		defaultValues: {
			scriptId: "",
			cron: "",
		} satisfies EvaluateScriptInput,
	});

	return (
		<Form {...form}>
			<form onSubmit={form.handleSubmit(props.onSubmit)} className="space-y-8">
				<FormField
					control={form.control}
					name="scriptId"
					render={({ field }) => (
						<FormItem>
							<FormLabel>スクリプトID</FormLabel>
							<FormControl>
								<Input placeholder="scriptId" {...field} />
							</FormControl>
						</FormItem>
					)}
				/>

				<FormField
					control={form.control}
					name="cron"
					render={({ field }) => (
						<FormItem>
							<FormLabel>cron</FormLabel>
							<FormControl>
								<Input
									placeholder="cron"
									{...field}
									value={field.value ?? undefined}
								/>
							</FormControl>
						</FormItem>
					)}
				/>

				<Button disabled={props.disabled} type="submit">
					作成
				</Button>
			</form>
		</Form>
	);
}
