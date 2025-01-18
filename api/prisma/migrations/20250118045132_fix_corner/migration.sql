/*
  Warnings:

  - You are about to drop the column `schema` on the `mails` table. All the data in the column will be lost.
  - Added the required column `mail_schema` to the `corners` table without a default value. This is not possible if the table is not empty.

*/
-- AlterTable
ALTER TABLE "corners" ADD COLUMN     "mail_schema" JSONB NOT NULL;

-- AlterTable
ALTER TABLE "mails" DROP COLUMN "schema";
