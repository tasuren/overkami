import { Component, createSignal, createContext, ParentProps } from "solid-js";

export enum Screen {
    Profiles = 1,
    Wallpapers = 2
}

const [screen, setScreen] = createSignal(Screen.Profiles);
export const ScreenContext = createContext({
    screen,
    setScreen
});

export const ScreenProvider: Component<ParentProps> = (props) => {
    return (
        <ScreenContext.Provider
            value={{
                screen,
                setScreen
            }}
        >
            {props.children}
        </ScreenContext.Provider>
    );
};
