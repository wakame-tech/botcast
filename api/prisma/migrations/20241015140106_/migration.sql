/*
  Warnings:

  - Added the required column `title` to the `scripts` table without a default value. This is not possible if the table is not empty.

*/
-- DropForeignKey
ALTER TABLE "episodes" DROP CONSTRAINT "episodes_script_id_fkey";

-- AlterTable
ALTER TABLE "scripts" ADD COLUMN     "title" TEXT NOT NULL;
