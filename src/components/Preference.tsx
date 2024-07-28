import { HiSolidCog6Tooth } from "solid-icons/hi";
import { Component, For, useContext } from "solid-js";
import { PreferenceContext, Theme, THEMES } from "~/lib/preference";

function capitalize(s: string): string {
    return s && s[0].toUpperCase() + s.slice(1);
}

const ThemeSelect: Component<{
    theme: () => Theme;
    setTheme: (theme: Theme) => void;
}> = (props) => {
    return (
        <>
            <label for="theme" class="pr-2">
                テーマ
            </label>
            <select
                name="theme"
                onchange={(e) =>
                    props.setTheme(
                        (e.target as HTMLSelectElement).value as Theme
                    )
                }
            >
                <For each={THEMES}>
                    {(theme) => (
                        <option value={theme} selected={props.theme() == theme}>
                            {capitalize(theme)}
                        </option>
                    )}
                </For>
            </select>
        </>
    );
};

const Preference: Component<{ ref: HTMLDialogElement }> = (props) => {
    const preference = useContext(PreferenceContext);

    return (
        <>
            <dialog
                class="
                    absolute top-0 left-0 bottom-0 right-0 m-auto w-5/6 h-5/6
                    rounded-lg
                    border-[0.5px] dark:border-zinc-700
                "
                ref={props.ref}
            >
                <form
                    method="dialog"
                    class="w-full h-full p-4 flex flex-col native-controls"
                >
                    <h1>設定</h1>

                    <div>
                        <ThemeSelect
                            theme={preference.theme}
                            setTheme={preference.setTheme}
                        />
                    </div>

                    <div class="mt-auto flex justify-end space-x-2">
                        <button type="submit" class="ml-auto mt-auto">
                            閉じる
                        </button>
                    </div>
                </form>
            </dialog>
        </>
    );
};

export default Preference;
