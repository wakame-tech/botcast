/*
  Warnings:

  - You are about to drop the column `audio_url` on the `episodes` table. All the data in the column will be lost.

*/
-- AlterTable
ALTER TABLE "episodes" DROP COLUMN "audio_url",
ADD COLUMN     "audioUrl" TEXT;
