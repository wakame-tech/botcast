/*
  Warnings:

  - Added the required column `icon` to the `podcasts` table without a default value. This is not possible if the table is not empty.

*/
-- AlterTable
ALTER TABLE "episodes" ADD COLUMN     "created_at" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP;

-- AlterTable
ALTER TABLE "podcasts" ADD COLUMN     "created_at" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
ADD COLUMN     "icon" VARCHAR(1) NOT NULL;
