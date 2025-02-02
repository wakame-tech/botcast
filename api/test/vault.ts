import { PrismaClient } from "prisma_client";
import {
  createSecret,
  deleteSecret,
  getSecret,
  listSecrets,
  updateSecret,
} from "../src/vault.ts";
import { assertEquals } from "jsr:@std/assert";

const removeAllSecrets = async (prisma: PrismaClient, userUuid: string) => {
  for (const { id } of await listSecrets(prisma, userUuid)) {
    await deleteSecret(prisma, userUuid, id);
  }
};

const cleanUp = (prisma: PrismaClient, userUuid: string) => ({
  [Symbol.asyncDispose]: () => removeAllSecrets(prisma, userUuid),
});

Deno.test("Create and update a secret", async () => {
  const prisma = new PrismaClient();
  const userUuid = "user";

  await removeAllSecrets(prisma, userUuid);
  await using _ = cleanUp(prisma, userUuid);

  const id = await createSecret(prisma, userUuid, "my_s3kre3t", "test");
  const res = await getSecret(prisma, userUuid, id);
  assertEquals(res?.name, `${userUuid}:test`);
  assertEquals(res?.decrypted_secret, "my_s3kre3t");

  await updateSecret(prisma, userUuid, id, "n3w_s3kret", "test2");
  const res2 = await getSecret(prisma, userUuid, id);
  assertEquals(res2?.name, `${userUuid}:test2`);
  assertEquals(res2?.decrypted_secret, "n3w_s3kret");
});

Deno.test("Multiple user can make same name secrets", async () => {
  const prisma = new PrismaClient();
  const user1Uuid = "user1";
  const user2Uuid = "user2";

  await removeAllSecrets(prisma, user1Uuid);
  await removeAllSecrets(prisma, user2Uuid);
  await using _1 = cleanUp(prisma, user1Uuid);
  await using _2 = cleanUp(prisma, user2Uuid);

  await createSecret(prisma, user1Uuid, "my_s3kre3t", "test");
  await createSecret(prisma, user2Uuid, "my_s3kre3t", "test");
});
