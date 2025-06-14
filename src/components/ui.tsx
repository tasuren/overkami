import { tv } from "tailwind-variants";

export const iconClass = tv({
  base: "stroke-dark dark:stroke-light",
});

export const buttonClass = tv({
  base: [
    "rounded-lg shadow-md",
    "w-fit",
    "cursor-pointer active:scale-95 transition",
  ],
  variants: {
    color: {
      primary: "bg-primary hover:bg-primary/70",
    },
    size: {
      sm: "h-8 px-3 py-1",
      lg: "h-10 px-4 py-2",
    },
    disabled: {
      true: "opacity-60 bg-primary pointer-events-none",
    },
  },
  defaultVariants: {
    color: "primary",
    size: "lg",
  },
});

export const iconButtonClass = tv({
  base: [
    "hover:bg-black/20 hover:dark:bg-white/10 rounded-lg",
    "w-fit h-auto p-2",
    "cursor-pointer active:scale-95 transition",
  ],
});

export const textMutedClass = tv({
  base: ["text-dark/60 dark:text-light/60"],
});

export const textInputClass = tv({
  base: [
    "h-10 p-2 rounded-lg",
    "border border-dark/40 dark:border-light/40",
    "bg-white/40 dark:bg-black/40",
  ],
  variants: {
    disabled: {
      true: "opacity-60 pointer-events-none",
    },
  },
});

export const selectClass = tv({
  variants: {
    disabled: {
      true: {
        base: "opacity-60 pointer-events-none",
      },
    },
  },
  slots: {
    base: [...textInputClass.base, "appearance-none cursor-pointer"],
    chevron: [
      "absolute right-3 top-1/2 -translate-y-1/2",
      "cursor-pointer pointer-events-none",
    ],
  },
});

export const fieldClass = tv({
  slots: {
    base: "flex flex-col gap-2",
    error: "text-dark/80 dark:text-light/80 text-sm",
  },
});

export const boxClass = tv({
  base: "p-4 rounded-lg border border-dark dark:border-light overflow-y-scroll",
});
