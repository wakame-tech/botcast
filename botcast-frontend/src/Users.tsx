import { trpc } from "./trpc";

export const Users: React.FC = () => {
  const usersQuery = trpc.users.useQuery();

  return <p>{JSON.stringify(usersQuery.data)}</p>;
};
