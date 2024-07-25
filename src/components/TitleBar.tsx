import { HiSolidCog6Tooth } from "solid-icons/hi";
import PreferenceDialog from "./Preference";
import { createMemo, createSignal } from "solid-js";

export function TitleBar() {
    const [isOpenState, setOpenState] = createSignal(false);
    const isOpen = createMemo(() => isOpenState());

    return (
        <>
            <div
                data-tauri-drag-region
                class="
                    w-30 flex-none h-[50px] p-2 gap-2 flex
                    justify-end place-content-center
                "
            >
                <button
                    type="button"
                    class="
                        hover:bg-dark/10 hover:dark:bg-light/10
                        aspect-square rounded-md
                        my-auto p-1
                    "
                    onclick={() => setOpenState(true)}
                >
                    <HiSolidCog6Tooth
                        size={22}
                        class="text-dark dark:text-light box-content"
                    />
                </button>
            </div>

            <hr class="w-full text-dark dark:text-light size-[1px]" />

            <PreferenceDialog isOpen={isOpen} />
        </>
    );
}
