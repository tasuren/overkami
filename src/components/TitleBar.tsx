import { HiOutlineCog6Tooth } from "solid-icons/hi";

import { Button } from "./ui/button";
import { Separator } from "./ui/separator";
import { Config } from "./Config";

export function TitleBar() {
    return (
        <>
            <div
                data-tauri-drag-region
                class="
                    grow w-30 h-[50px] px-2 gap-2 flex
                    justify-end place-content-center
                "
            >
                <Config className="aspect-square my-auto" />
            </div>

            <Separator class="w-full" />
        </>
    );
}
