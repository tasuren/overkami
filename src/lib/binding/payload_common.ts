export type ErrorContext =
  | string
  | {
      message: string;
      detail: string;
    };
