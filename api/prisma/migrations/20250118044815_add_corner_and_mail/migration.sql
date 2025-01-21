/*
  Warnings:

  - You are about to drop the `comments` table. If the table is not empty, all the data it contains will be lost.

*/
-- DropForeignKey
ALTER TABLE "comments" DROP CONSTRAINT "comments_episode_id_fkey";

-- DropForeignKey
ALTER TABLE "comments" DROP CONSTRAINT "comments_user_id_fkey";

-- AlterTable
ALTER TABLE "scripts" ADD COLUMN     "arguments" JSONB NOT NULL DEFAULT '{}';

-- DropTable
DROP TABLE "comments";

-- CreateTable
CREATE TABLE "corners" (
    "id" UUID NOT NULL DEFAULT gen_random_uuid(),
    "title" TEXT NOT NULL,
    "description" TEXT NOT NULL,
    "requesting_mail" BOOLEAN NOT NULL,
    "user_id" UUID NOT NULL,

    CONSTRAINT "corners_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "mails" (
    "id" UUID NOT NULL DEFAULT gen_random_uuid(),
    "name" TEXT NOT NULL,
    "schema" JSONB NOT NULL,
    "body" JSONB NOT NULL,
    "user_id" UUID NOT NULL,
    "corner_id" UUID NOT NULL,

    CONSTRAINT "mails_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "_CornerToPodcast" (
    "A" UUID NOT NULL,
    "B" UUID NOT NULL
);

-- CreateIndex
CREATE UNIQUE INDEX "_CornerToPodcast_AB_unique" ON "_CornerToPodcast"("A", "B");

-- CreateIndex
CREATE INDEX "_CornerToPodcast_B_index" ON "_CornerToPodcast"("B");

-- AddForeignKey
ALTER TABLE "corners" ADD CONSTRAINT "corners_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "users"("id") ON DELETE CASCADE ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "mails" ADD CONSTRAINT "mails_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "users"("id") ON DELETE CASCADE ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "mails" ADD CONSTRAINT "mails_corner_id_fkey" FOREIGN KEY ("corner_id") REFERENCES "corners"("id") ON DELETE CASCADE ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "_CornerToPodcast" ADD CONSTRAINT "_CornerToPodcast_A_fkey" FOREIGN KEY ("A") REFERENCES "corners"("id") ON DELETE CASCADE ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "_CornerToPodcast" ADD CONSTRAINT "_CornerToPodcast_B_fkey" FOREIGN KEY ("B") REFERENCES "podcasts"("id") ON DELETE CASCADE ON UPDATE CASCADE;
