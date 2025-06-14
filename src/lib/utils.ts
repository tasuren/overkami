export function cl(...classes: (string | boolean | undefined | null)[]) {
  return classes.filter(Boolean).join(" ");
}

export type OptionalizeAll<T> = {
  [P in keyof T]?: T[P];
};
