import { PrismaClient } from "@prisma/client";
import {
    createSecret,
    deleteSecret,
    getSecret,
    listSecrets,
    updateSecret,
} from "../src/vault.ts";
import { assertEquals } from "jsr:@std/assert";

const cleanUp = (prisma: PrismaClient, userUuid: string) => ({
    [Symbol.asyncDispose]: async () => {
        for (const { id } of await listSecrets(prisma, userUuid)) {
            await deleteSecret(prisma, userUuid, id);
        }
    },
});

Deno.test("Supabase vault", async () => {
    const prisma = new PrismaClient();
    const userUuid = "user";

    await using _ = cleanUp(prisma, userUuid);

    const id = await createSecret(prisma, userUuid, "my_s3kre3t", "test");
    const res = await getSecret(prisma, userUuid, id);
    assertEquals(res?.name, "test");
    assertEquals(res?.decrypted_secret, "my_s3kre3t");

    await updateSecret(prisma, userUuid, id, "n3w_s3kret", "test2");
    const res2 = await getSecret(prisma, userUuid, id);
    assertEquals(res2?.name, "test2");
    assertEquals(res2?.decrypted_secret, "n3w_s3kret");
});
