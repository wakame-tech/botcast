/*
  Warnings:

  - You are about to drop the `_CornerToPodcast` table. If the table is not empty, all the data it contains will be lost.
  - Added the required column `podcast_id` to the `corners` table without a default value. This is not possible if the table is not empty.

*/
-- DropForeignKey
ALTER TABLE "_CornerToPodcast" DROP CONSTRAINT "_CornerToPodcast_A_fkey";

-- DropForeignKey
ALTER TABLE "_CornerToPodcast" DROP CONSTRAINT "_CornerToPodcast_B_fkey";

-- AlterTable
ALTER TABLE "corners" ADD COLUMN     "podcast_id" UUID NOT NULL;

-- DropTable
DROP TABLE "_CornerToPodcast";

-- AddForeignKey
ALTER TABLE "corners" ADD CONSTRAINT "corners_podcast_id_fkey" FOREIGN KEY ("podcast_id") REFERENCES "podcasts"("id") ON DELETE CASCADE ON UPDATE CASCADE;
