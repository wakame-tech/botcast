-- CreateTable
CREATE TABLE "Episode" (
    "id" UUID NOT NULL DEFAULT gen_random_uuid(),
    "title" TEXT NOT NULL,
    "content" TEXT NOT NULL,
    "audio_url" TEXT,

    CONSTRAINT "Episode_pkey" PRIMARY KEY ("id")
);
