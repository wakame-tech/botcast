/*
  Warnings:

  - You are about to drop the column `audioUrl` on the `episodes` table. All the data in the column will be lost.
  - You are about to drop the column `userId` on the `episodes` table. All the data in the column will be lost.
  - You are about to drop the column `userId` on the `tasks` table. All the data in the column will be lost.

*/
-- DropForeignKey
ALTER TABLE "episodes" DROP CONSTRAINT "episodes_userId_fkey";

-- DropForeignKey
ALTER TABLE "tasks" DROP CONSTRAINT "tasks_userId_fkey";

-- AlterTable
ALTER TABLE "episodes" DROP COLUMN "audioUrl",
DROP COLUMN "userId",
ADD COLUMN     "audio_url" TEXT,
ADD COLUMN     "user_id" UUID;

-- AlterTable
ALTER TABLE "tasks" DROP COLUMN "userId",
ADD COLUMN     "user_id" UUID;

-- AddForeignKey
ALTER TABLE "episodes" ADD CONSTRAINT "episodes_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "users"("id") ON DELETE SET NULL ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "tasks" ADD CONSTRAINT "tasks_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "users"("id") ON DELETE SET NULL ON UPDATE CASCADE;
