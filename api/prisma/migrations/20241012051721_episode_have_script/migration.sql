/*
  Warnings:

  - You are about to drop the column `script_url` on the `episodes` table. All the data in the column will be lost.
  - Added the required column `script_id` to the `episodes` table without a default value. This is not possible if the table is not empty.
  - Added the required column `user_id` to the `scripts` table without a default value. This is not possible if the table is not empty.

*/
-- AlterTable
ALTER TABLE "episodes" DROP COLUMN "script_url",
ADD COLUMN     "manuscript" JSONB,
ADD COLUMN     "script_id" UUID NOT NULL;

-- AlterTable
ALTER TABLE "scripts" ADD COLUMN     "user_id" UUID NOT NULL;

-- AddForeignKey
ALTER TABLE "episodes" ADD CONSTRAINT "episodes_script_id_fkey" FOREIGN KEY ("script_id") REFERENCES "scripts"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "scripts" ADD CONSTRAINT "scripts_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "users"("id") ON DELETE CASCADE ON UPDATE CASCADE;
