import { PrismaClient } from "@prisma/client";

interface CreateSecretResponse {
    // UUID of the created secret
    create_secret: string;
}

export const createSecret = async (
    prisma: PrismaClient,
    userUuid: string,
    secret: string,
    name: string,
): Promise<string> => {
    // NOTE: description used by storing user uuid
    prisma.$queryRaw;
    const res = await prisma.$queryRaw<
        CreateSecretResponse[]
    >`select vault.create_secret(${secret}, ${name}, ${userUuid});`;
    return res[0].create_secret;
};

interface SelectSecretResponse {
    id: string;
    name: string;
    description: string;
    secret: string;
    decrypted_secret: string;
    key_id: string;
    created_at: string;
    updated_at: string;
}

export const listSecrets = (
    prisma: PrismaClient,
    userUuid: string,
): Promise<SelectSecretResponse[]> => {
    return prisma.$queryRaw<
        SelectSecretResponse[]
    >`select * from vault.decrypted_secrets where description = ${userUuid};`;
};

export const getSecret = async (
    prisma: PrismaClient,
    userUuid: string,
    id: string,
): Promise<SelectSecretResponse | null> => {
    const res = await prisma.$queryRaw<
        SelectSecretResponse[]
    >`select * from vault.decrypted_secrets where description = ${userUuid} and id = ${id}::uuid;`;
    return res.length === 1 ? res[0] : null;
};

export const updateSecret = async (
    prisma: PrismaClient,
    userUuid: string,
    id: string,
    secret: string,
    name: string,
): Promise<void> => {
    if (await getSecret(prisma, userUuid, id) === null) {
        throw new Error("Secret not found");
    }
    await prisma.$executeRaw`select
      vault.update_secret(
        ${id}::uuid,
        ${secret},
        ${name},
        ${userUuid}
      );
    `;
};

export const deleteSecret = async (
    prisma: PrismaClient,
    userUuid: string,
    id: string,
): Promise<void> => {
    await prisma
        .$executeRaw`delete from vault.secrets where description = ${userUuid} and id = ${id}::uuid;`;
};
