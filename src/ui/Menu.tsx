import { Component, For, useContext } from "solid-js";

import { Screen, ScreenContext } from "../screen";


const SCREENS = {
    "🗂️ Profiles": Screen.Profiles,
    "🌌 Wallpapers": Screen.Wallpapers
} as const;
type ScreenTitle = keyof typeof SCREENS;
const { screen, setScreen } = useContext(ScreenContext);

let currentScreenTitle = "";
const MenuButton: Component<{
    title: ScreenTitle;
    className: string;
}> = ({ title, className }) => {
    if (!currentScreenTitle && title) currentScreenTitle = title;

    return (
        <button
            class={`${className} ${styles.menuButton}`}
            type="button"
            onclick={() => {
                currentScreenTitle = title;
                setScreen(SCREENS[title]);
            }}
            disabled={screen() == SCREENS[title]}
        >
            {title}
        </button>
    );
};

export const Menu: Component<{
    className: string;
}> = ({ className = "" }) => {
    return (
        <For each={Object.keys(SCREENS)}>
            {(title) => (
                <MenuButton
                    title={title as ScreenTitle}
                    className={className}
                />
            )}
        </For>
    );
};
