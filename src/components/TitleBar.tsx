import { HiOutlineCog6Tooth } from "solid-icons/hi";

export function TitleBar() {
    return (
        <>
            <div
                data-tauri-drag-region
                class="
                    w-30 flex-none h-[50px] px-2 gap-2 flex
                    justify-end place-content-center
                "
            >
                <HiOutlineCog6Tooth
                    size={26}
                    class="text-dark dark:text-light my-auto p-1 box-content"
                />
            </div>
        </>
    );
}
