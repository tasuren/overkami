import { Accessor } from "solid-js";
import { Component, useContext } from "solid-js";
import { PreferenceContext } from "~/lib/preference";

function capitalize(s: string): string {
    return s && s[0].toUpperCase() + s.slice(1);
}

const ThemeSelect: Component = () => {
    return <></>;
};

const PreferenceDialog: Component<{ isOpen: Accessor<boolean> }> = ({
    isOpen
}) => {
    const preference = useContext(PreferenceContext);

    return (
        <dialog
            open={isOpen()}
            class="absolute top-0 left-0 bottom-0 right-0 m-auto"
        >
            <p>あいうえお</p>
            <form method="dialog">
                <button>OK</button>
            </form>
        </dialog>
    );
};

export default PreferenceDialog;
