set dotenv-load

worker:
  (cd botcast-job-worker && deno run -A ./src/index.ts)

worker_test:
  (cd botcast-job-worker && deno run -A ./test/index.ts)
