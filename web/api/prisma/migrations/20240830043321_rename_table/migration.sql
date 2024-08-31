/*
  Warnings:

  - You are about to drop the `Episode` table. If the table is not empty, all the data it contains will be lost.

*/
-- DropTable
DROP TABLE "Episode";

-- CreateTable
CREATE TABLE "episodes" (
    "id" UUID NOT NULL DEFAULT gen_random_uuid(),
    "title" TEXT NOT NULL,
    "content" TEXT NOT NULL,
    "audio_url" TEXT,

    CONSTRAINT "episodes_pkey" PRIMARY KEY ("id")
);
