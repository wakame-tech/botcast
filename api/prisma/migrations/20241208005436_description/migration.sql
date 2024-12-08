-- AlterTable
ALTER TABLE "episodes" ADD COLUMN     "description" TEXT,
ADD COLUMN     "duration_sec" INTEGER;

-- AlterTable
ALTER TABLE "podcasts" ADD COLUMN     "description" TEXT;

-- AlterTable
ALTER TABLE "scripts" ADD COLUMN     "description" TEXT;
