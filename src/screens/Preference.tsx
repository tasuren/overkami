import { Component, useContext } from "solid-js";

import { HiOutlineCog6Tooth } from "solid-icons/hi";
import { PreferenceContext } from "~/lib/preference";

function capitalize(s: string): string {
    return s && s[0].toUpperCase() + s.slice(1);
}

const ThemeSelect: Component = () => {
    return (
        <button type="button">
            <HiOutlineCog6Tooth />
        </button>
    );
};

const PreferenceScreen: Component = () => {
    const preference = useContext(PreferenceContext);

    return (
        <>
            <ThemeSelect />
        </>
    );
};

export default PreferenceScreen;
