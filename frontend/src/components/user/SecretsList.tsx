import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useState } from "react";

interface Secret {
	id: string;
	name: string;
}

export interface NewSecret {
	name: string;
	value: string;
}

interface SecretListProps {
	secrets: Secret[];
	onSubmit: (newSecrets: NewSecret[], deletionIds: string[]) => Promise<void>;
}

export function SecretList(props: SecretListProps) {
	const [newSecrets, setNewSecrets] = useState<NewSecret[]>([]);
	const [deletionIds, setDeletionIds] = useState<string[]>([]);
	const [name, setName] = useState("");
	const [value, setValue] = useState("");

	const addColumn = () => {
		if (name && value) {
			setNewSecrets([...newSecrets, { name, value }]);
			setName("");
			setValue("");
		}
	};

	const toggleDeleteSecret = (id: string) => {
		if (deletionIds.includes(id)) {
			setDeletionIds(deletionIds.filter((i) => i !== id));
		} else {
			setDeletionIds([...deletionIds, id]);
		}
	};

	const deleteColumn = (index: number) => {
		setNewSecrets(newSecrets.filter((_, i) => i !== index));
	};

	const handleSubmit = () => {
		props.onSubmit(newSecrets, deletionIds).then(() => {
			setNewSecrets([]);
			setDeletionIds([]);
		});
	};

	return (
		<>
			<ul className="space-y-2 p-0">
				{props.secrets.map((secret) => (
					<li
						key={secret.id}
						className="flex items-center bg-background border rounded"
					>
						<div className="flex-grow">
							<div className="flex gap-2">
								<span
									className={`
                                    ${deletionIds.includes(secret.id) ? "line-through" : ""}
                                `}
								>
									{secret.name}
								</span>
							</div>
						</div>
						{deletionIds.includes(secret.id) ? (
							<Button onClick={() => toggleDeleteSecret(secret.id)}>
								取消
							</Button>
						) : (
							<Button
								className="bg-red-500"
								onClick={() => toggleDeleteSecret(secret.id)}
							>
								削除
							</Button>
						)}
					</li>
				))}
				{newSecrets.map((secret, i) => (
					<li
						key={secret.name}
						className="flex items-center bg-background border rounded"
					>
						<div className="flex-grow">
							<div className="flex gap-2">
								<span className="">{secret.name}</span>
							</div>
						</div>
						<Button onClick={() => deleteColumn(i)}>削除</Button>
					</li>
				))}
			</ul>

			<div className="flex gap-2 mb-4">
				<Input
					type="text"
					placeholder="KEY"
					value={name}
					onChange={(e) => setName(e.target.value)}
				/>
				<Input
					type="text"
					placeholder="VALUE"
					value={value}
					onChange={(e) => setValue(e.target.value)}
				/>
				<Button onClick={addColumn} aria-label="追加">
					追加
				</Button>
			</div>

			{(newSecrets.length !== 0 || deletionIds.length !== 0) && (
				<div className="mt-4">
					<Button
						className="bg-blue-400"
						onClick={handleSubmit}
						aria-label="反映"
					>
						反映({newSecrets.length} New, {deletionIds.length} Delete)
					</Button>
				</div>
			)}
		</>
	);
}
