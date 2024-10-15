/*
  Warnings:

  - Added the required column `script_id` to the `podcasts` table without a default value. This is not possible if the table is not empty.

*/
-- AlterTable
ALTER TABLE "podcasts" ADD COLUMN     "script_id" UUID NOT NULL;

-- AddForeignKey
ALTER TABLE "episodes" ADD CONSTRAINT "episodes_script_id_fkey" FOREIGN KEY ("script_id") REFERENCES "scripts"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "podcasts" ADD CONSTRAINT "podcasts_script_id_fkey" FOREIGN KEY ("script_id") REFERENCES "scripts"("id") ON DELETE RESTRICT ON UPDATE CASCADE;
