/*
  Warnings:

  - You are about to drop the column `cron` on the `scripts` table. All the data in the column will be lost.

*/
-- AlterTable
ALTER TABLE "scripts" DROP COLUMN "cron";

-- AlterTable
ALTER TABLE "tasks" ADD COLUMN     "cron" TEXT;
