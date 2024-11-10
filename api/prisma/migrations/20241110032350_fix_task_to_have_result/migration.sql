/*
  Warnings:

  - You are about to drop the column `script_id` on the `episodes` table. All the data in the column will be lost.
  - You are about to drop the column `result` on the `scripts` table. All the data in the column will be lost.
  - Added the required column `manuscript` to the `episodes` table without a default value. This is not possible if the table is not empty.

*/
-- DropForeignKey
ALTER TABLE "episodes" DROP CONSTRAINT "episodes_script_id_fkey";

-- AlterTable
ALTER TABLE "episodes" DROP COLUMN "script_id",
ADD COLUMN     "manuscript" JSONB NOT NULL;

-- AlterTable
ALTER TABLE "scripts" DROP COLUMN "result";

-- AlterTable
ALTER TABLE "tasks" ADD COLUMN     "executed_finished_at" TIMESTAMPTZ(3),
ADD COLUMN     "result" JSONB;
