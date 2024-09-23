-- AlterTable
ALTER TABLE "episodes" ALTER COLUMN "created_at" DROP DEFAULT,
ALTER COLUMN "created_at" SET DATA TYPE TEXT;

-- AlterTable
ALTER TABLE "podcasts" ALTER COLUMN "created_at" DROP DEFAULT,
ALTER COLUMN "created_at" SET DATA TYPE TEXT;
