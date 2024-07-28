import { Component } from "solid-js";
import { HiSolidCog6Tooth } from "solid-icons/hi";

import Preference from "./Preference";

const PreferenceButton: Component<{ dialog: () => HTMLDialogElement }> = ({
    dialog
}) => {
    return (
        <button
            type="button"
            class="
            with-preflight
            hover:bg-dark/10 hover:dark:bg-light/10
            aspect-square rounded-md
            my-auto p-1
        "
            onclick={() => dialog().showModal()}
        >
            <HiSolidCog6Tooth
                size={22}
                class="text-dark dark:text-light box-content"
            />
        </button>
    );
};

export function TitleBar() {
    let dialog!: HTMLDialogElement;

    return (
        <>
            <div
                data-tauri-drag-region
                class="
                    w-30 flex-none h-[45px] p-2 gap-2 flex
                    justify-end place-content-center
                "
            >
                <PreferenceButton dialog={() => dialog} />
            </div>

            <hr class="w-full text-dark dark:text-light size-[1px]" />

            <Preference ref={dialog} />
        </>
    );
}
