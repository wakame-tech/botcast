/*
  Warnings:

  - You are about to drop the `Botcast` table. If the table is not empty, all the data it contains will be lost.
  - You are about to drop the `Script` table. If the table is not empty, all the data it contains will be lost.
  - You are about to drop the `Task` table. If the table is not empty, all the data it contains will be lost.

*/
-- DropTable
DROP TABLE "Botcast";

-- DropTable
DROP TABLE "Script";

-- DropTable
DROP TABLE "Task";

-- CreateTable
CREATE TABLE "botcast" (
    "id" TEXT NOT NULL,
    "title" TEXT NOT NULL,
    "description" TEXT NOT NULL,
    "date" TIMESTAMP(3) NOT NULL,
    "url" TEXT NOT NULL,

    CONSTRAINT "botcast_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "script" (
    "id" TEXT NOT NULL,
    "script" JSONB NOT NULL,

    CONSTRAINT "script_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "task" (
    "id" TEXT NOT NULL,
    "status" TEXT NOT NULL,
    "body" TEXT NOT NULL,
    "date" TIMESTAMP(3) NOT NULL,

    CONSTRAINT "task_pkey" PRIMARY KEY ("id")
);
