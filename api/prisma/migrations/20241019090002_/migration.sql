/*
  Warnings:

  - The `created_at` column on the `comments` table would be dropped and recreated. This will lead to data loss if there is data in the column.
  - The `created_at` column on the `episodes` table would be dropped and recreated. This will lead to data loss if there is data in the column.
  - The `created_at` column on the `podcasts` table would be dropped and recreated. This will lead to data loss if there is data in the column.

*/
-- AlterTable
ALTER TABLE "comments" DROP COLUMN "created_at",
ADD COLUMN     "created_at" TIMESTAMPTZ(3) NOT NULL DEFAULT CURRENT_TIMESTAMP;

-- AlterTable
ALTER TABLE "episodes" DROP COLUMN "created_at",
ADD COLUMN     "created_at" TIMESTAMPTZ(3) NOT NULL DEFAULT CURRENT_TIMESTAMP;

-- AlterTable
ALTER TABLE "podcasts" DROP COLUMN "created_at",
ADD COLUMN     "created_at" TIMESTAMPTZ(3) NOT NULL DEFAULT CURRENT_TIMESTAMP;

-- AlterTable
ALTER TABLE "tasks" ADD COLUMN     "execute_after" TIMESTAMPTZ(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
ADD COLUMN     "executed_at" TIMESTAMPTZ(3);
