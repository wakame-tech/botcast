/*
  Warnings:

  - Added the required column `podcast_id` to the `episodes` table without a default value. This is not possible if the table is not empty.

*/
-- AlterTable
ALTER TABLE "episodes" ADD COLUMN     "podcast_id" UUID NOT NULL;

-- CreateTable
CREATE TABLE "podcasts" (
    "id" UUID NOT NULL DEFAULT gen_random_uuid(),
    "title" TEXT NOT NULL,
    "user_id" UUID,

    CONSTRAINT "podcasts_pkey" PRIMARY KEY ("id")
);

-- AddForeignKey
ALTER TABLE "episodes" ADD CONSTRAINT "episodes_podcast_id_fkey" FOREIGN KEY ("podcast_id") REFERENCES "podcasts"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "podcasts" ADD CONSTRAINT "podcasts_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "users"("id") ON DELETE SET NULL ON UPDATE CASCADE;
