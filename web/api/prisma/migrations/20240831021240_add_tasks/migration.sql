-- CreateEnum
CREATE TYPE "task_status" AS ENUM ('PENDING', 'RUNNING', 'COMPLETED', 'FAILED');

-- CreateTable
CREATE TABLE "tasks" (
    "id" UUID NOT NULL DEFAULT gen_random_uuid(),
    "status" "task_status" NOT NULL,
    "args" JSONB NOT NULL,

    CONSTRAINT "tasks_pkey" PRIMARY KEY ("id")
);
