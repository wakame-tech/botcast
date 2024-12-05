/*
  Warnings:

  - You are about to drop the column `cron` on the `podcasts` table. All the data in the column will be lost.
  - You are about to drop the column `script_id` on the `podcasts` table. All the data in the column will be lost.

*/
-- DropForeignKey
ALTER TABLE "podcasts" DROP CONSTRAINT "podcasts_script_id_fkey";

-- AlterTable
ALTER TABLE "podcasts" DROP COLUMN "cron",
DROP COLUMN "script_id";

-- AlterTable
ALTER TABLE "scripts" ADD COLUMN     "cron" TEXT;
