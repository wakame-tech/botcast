-- CreateTable
CREATE TABLE "scripts" (
    "id" UUID NOT NULL DEFAULT gen_random_uuid(),
    "template" JSONB NOT NULL,

    CONSTRAINT "scripts_pkey" PRIMARY KEY ("id")
);
