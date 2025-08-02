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
import { type ScriptInput, ScriptInputSchema } from "@/lib/api_client";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";

interface ScriptFormProps {
	disabled?: boolean;
	values?: ScriptInput;
	onSubmit: (values: ScriptInput) => void;
}

export function ScriptForm(props: ScriptFormProps) {
	const form = useForm<ScriptInput>({
		resolver: zodResolver(ScriptInputSchema),
		defaultValues:
			props.values ??
			({
				title: "NewScript",
				description: "",
				template: "",
			} satisfies ScriptInput),
	});

	return (
		<Form {...form}>
			<form onSubmit={form.handleSubmit(props.onSubmit)} className="space-y-8">
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
				<Button disabled={props.disabled} type="submit">
					{props.values ? "更新" : "作成"}
				</Button>
			</form>
		</Form>
	);
}
