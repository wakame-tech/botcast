export const collectPromises = async <T>(
  promises: (() => Promise<T>)[]
): Promise<T[]> => {
  const res = [];
  for (const p of promises) {
    res.push(await p());
  }
  return res;
};

export const andThen =
  <T, R>(f: (a: T) => Promise<R>): ((pa: Promise<T>) => Promise<R>) =>
  (pa) =>
    pa.then((a) => f(a));
