export const deepcopy = <T,>(v: T): T => {
  if (Array.isArray(v)) {
    return v.map(deepcopy) as T;
  } else if (v instanceof Object) {
    return Object.fromEntries(Object.entries(v).map(([k, v]) => [k, deepcopy(v)])) as T
  } else {
    return v
  }
}

export const last = <T,>(array: T[]) => array[array.length - 1];

// export const lastn = (n: number) => <T,>(array: T[]) => array.slice(n)

export const nullIfEmpty = <T,>(array: T[]) => (array.length === 0 ? null : array);

export const dropLast = <T,>(array: T[]) => array.slice(0, -1);

// export const without = (array, exclude) => array.filter((item) => item !== exclude);

/* isn't this just the ?? operator? */
export const coalesce = <T,>(t: T | null | undefined, fallback: () => T): T =>
  t === null || t === undefined ? fallback() : t;

export const appends =
  <T,>(item: T) =>
  <U,>(array: (T | U)[]) =>
    [...array, item];

export const removesAt =
  (...indexes: number[] /* in ascending order */) =>
  <T,>(array: T[]) =>
    [0, ...indexes.map((n) => n + 1)].flatMap((n, i) =>
      array.slice(n, indexes[i])
    );

export const getsAt =
  (...indexes: number[]) =>
  <T,>(array: T[]) =>
    indexes.map((i) => array[i]);

export const insertsAt =
  (index: number) =>
  <T,>(insert: T[]) =>
  (array: T[]) =>
    index < array.length
      ? [...array.slice(0, index), ...insert, ...array.slice(index)]
      : [...array, ...insert];

export const setrattr =
  (target: any, props: (number | string)[], value: any) =>
  (dropLast(props).reduce((t, p) => t[p], target)[last(props)] = value)

export const donotwant = (n: never): never => n;
