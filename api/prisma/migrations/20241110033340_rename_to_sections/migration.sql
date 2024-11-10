/*
  Warnings:

  - You are about to drop the column `manuscript` on the `episodes` table. All the data in the column will be lost.
  - Added the required column `sections` to the `episodes` table without a default value. This is not possible if the table is not empty.

*/
-- AlterTable
ALTER TABLE "episodes" DROP COLUMN "manuscript",
ADD COLUMN     "sections" JSONB NOT NULL;
